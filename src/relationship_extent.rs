// SPDX-License-Identifier: AGPL-3.0-or-later
//! Admission and ordered reads for model supplied navigation extents and population evidence.

use alloc::vec::Vec;
use core::ptr;

use quire_exact::{EffectiveId, ObjectReference};

use crate::declaration::{
    reserve_elements, GlobalReservations, OrderedIndex, ReservationPolicy, StorageFailure,
    StorageRequest, StorageUnit, TypeEnvironment,
};

/// The side of a navigation relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndSide {
    /// The source end.
    Source,
    /// The target end.
    Target,
}

/// The authenticated type and uniqueness flag supplied for one end.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationshipEnd {
    /// The declared effective object type.
    pub object_type: EffectiveId,
    /// Whether this end declares uniqueness.
    pub unique: bool,
}

/// One model supplied navigation relationship.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationRelationship<K> {
    /// Caller owned relationship identity.
    pub key: K,
    /// Source end declaration.
    pub source: RelationshipEnd,
    /// Target end declaration.
    pub target: RelationshipEnd,
}

/// Why a navigation table could not be admitted.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TableRefusal<K> {
    /// A key occurs more than once.
    #[error("duplicate navigation relationship")]
    DuplicateRelationshipKey(K),
    /// An end type is absent from the admitted model.
    #[error("unknown navigation relationship end type")]
    UnknownEndType {
        /// Relationship identity.
        relationship: K,
        /// Refused end.
        side: EndSide,
        /// The unknown declared type.
        object_type: EffectiveId,
    },
    /// A representable reservation was denied.
    #[error("navigation table allocation denied: {0:?}")]
    Allocation(StorageRequest),
    /// Reservation size or representation overflowed.
    #[error("navigation table capacity exceeded: {0:?}")]
    Capacity(StorageRequest),
}

/// An admitted table of navigation relationships keyed by the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationRelationshipTable<K: Ord + Clone> {
    entries: OrderedIndex<K, (RelationshipEnd, RelationshipEnd)>,
}

impl<K: Ord + Clone> NavigationRelationshipTable<K> {
    /// Admit an entire table; repeated keys precede unknown source and target types.
    pub fn new(
        types: &TypeEnvironment,
        declarations: impl IntoIterator<Item = NavigationRelationship<K>>,
    ) -> Result<Self, TableRefusal<K>> {
        Self::with_reservation(types, declarations, &GlobalReservations)
    }

    fn with_reservation<R: ReservationPolicy>(
        types: &TypeEnvironment,
        declarations: impl IntoIterator<Item = NavigationRelationship<K>>,
        reservation: &R,
    ) -> Result<Self, TableRefusal<K>> {
        let mut entries = OrderedIndex::default();
        for declaration in declarations {
            let NavigationRelationship {
                key,
                source,
                target,
            } = declaration;
            if entries.contains_key(&key) {
                return Err(TableRefusal::DuplicateRelationshipKey(key));
            }
            for (side, end) in [(EndSide::Source, source), (EndSide::Target, target)] {
                if types.object_type(end.object_type).is_none() {
                    return Err(TableRefusal::UnknownEndType {
                        relationship: key,
                        side,
                        object_type: end.object_type,
                    });
                }
            }
            entries
                .try_insert(key, (source, target), reservation)
                .map_err(table_storage)?;
        }
        Ok(Self { entries })
    }

    /// The source and target declarations for `key`.
    pub fn ends(&self, key: &K) -> Option<(RelationshipEnd, RelationshipEnd)> {
        self.entries.get(key).copied()
    }
}

fn table_storage<K>(failure: StorageFailure) -> TableRefusal<K> {
    match failure {
        StorageFailure::Allocation(request) => TableRefusal::Allocation(request),
        StorageFailure::Capacity(request) => TableRefusal::Capacity(request),
    }
}

/// One supplied relationship extent; instances retain their supplied order until admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipExtent<K> {
    /// Relationship identity.
    pub relationship: K,
    /// Whether enumeration of this relationship was complete.
    pub complete: bool,
    /// Full source and target reference pairs.
    pub instances: Vec<(ObjectReference, ObjectReference)>,
}

/// Whether a relationship was listed, and whether its listed extent is complete.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtentState {
    /// No extent was supplied.
    Unlisted,
    /// A listed extent has incomplete evidence.
    Incomplete,
    /// A listed extent has complete evidence, possibly with no instances.
    Complete,
}

/// Why an extent sequence could not be admitted.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ExtentRefusal<K> {
    /// Condition 1: the key is absent from the navigation table.
    #[error("not a navigation relationship")]
    NotNavigationRelationship {
        /// Relationship identity.
        relationship: K,
        /// Zero based extent index.
        extent_index: usize,
    },
    /// Condition 2: another extent in this call already names the relationship.
    #[error("duplicate extent")]
    DuplicateExtent {
        /// Relationship identity.
        relationship: K,
        /// Zero based extent index.
        extent_index: usize,
    },
    /// Condition 3: an actual reference type does not conform to its declared end.
    #[error("relationship end type mismatch")]
    EndTypeMismatch {
        /// Relationship identity.
        relationship: K,
        /// Zero based extent index.
        extent_index: usize,
        /// Zero based instance index.
        instance_index: usize,
        /// Refused end.
        side: EndSide,
        /// Type carried by the full reference.
        actual_type: EffectiveId,
        /// Authenticated declared end type.
        declared_type: EffectiveId,
    },
    /// Condition 4: a repeated full triple when either end is unique.
    #[error("duplicate relationship triple")]
    DuplicateTriple {
        /// Relationship identity.
        relationship: K,
        /// Zero based extent index.
        extent_index: usize,
        /// Zero based instance index.
        instance_index: usize,
    },
    /// A representable reservation was denied.
    #[error("extent allocation denied: {0:?}")]
    Allocation(StorageRequest),
    /// Reservation size or representation overflowed.
    #[error("extent capacity exceeded: {0:?}")]
    Capacity(StorageRequest),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StoredExtent {
    complete: bool,
    links: Vec<(ObjectReference, ObjectReference)>,
    inverse: Vec<(ObjectReference, ObjectReference)>,
}

/// One transactionally admitted sequence of relationship extents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipExtentStore<K: Ord + Clone> {
    extents: OrderedIndex<K, StoredExtent>,
}

impl<K: Ord + Clone> RelationshipExtentStore<K> {
    /// Admit a sequence in one call, stopping at the first condition failure.
    pub fn new(
        types: &TypeEnvironment,
        table: &NavigationRelationshipTable<K>,
        extents: impl IntoIterator<Item = RelationshipExtent<K>>,
    ) -> Result<Self, ExtentRefusal<K>> {
        Self::with_reservation(types, table, extents, &GlobalReservations)
    }

    fn with_reservation<R: ReservationPolicy>(
        types: &TypeEnvironment,
        table: &NavigationRelationshipTable<K>,
        extents: impl IntoIterator<Item = RelationshipExtent<K>>,
        reservation: &R,
    ) -> Result<Self, ExtentRefusal<K>> {
        let mut admitted = OrderedIndex::default();
        for (extent_index, extent) in extents.into_iter().enumerate() {
            let RelationshipExtent {
                relationship,
                complete,
                instances,
            } = extent;
            let Some((source_end, target_end)) = table.ends(&relationship) else {
                return Err(ExtentRefusal::NotNavigationRelationship {
                    relationship,
                    extent_index,
                });
            };
            if admitted.contains_key(&relationship) {
                return Err(ExtentRefusal::DuplicateExtent {
                    relationship,
                    extent_index,
                });
            }
            let mut first_mismatch = None;
            for (instance_index, (source, target)) in instances.iter().enumerate() {
                for (side, actual, declared) in [
                    (
                        EndSide::Source,
                        source.object_type(),
                        source_end.object_type,
                    ),
                    (
                        EndSide::Target,
                        target.object_type(),
                        target_end.object_type,
                    ),
                ] {
                    if types.object_type(actual).is_none() || !types.conforms(actual, declared) {
                        first_mismatch = Some((
                            instance_index,
                            ExtentRefusal::EndTypeMismatch {
                                relationship: relationship.clone(),
                                extent_index,
                                instance_index,
                                side,
                                actual_type: actual,
                                declared_type: declared,
                            },
                        ));
                        break;
                    }
                }
                if first_mismatch.is_some() {
                    break;
                }
            }
            // Only the valid prefix can contain a duplicate that precedes the first mismatch.
            let checked_len = first_mismatch
                .as_ref()
                .map_or(instances.len(), |(instance_index, _)| *instance_index);
            if source_end.unique || target_end.unique {
                let mut triples = Vec::new();
                if let Err(storage) = reserve_elements(&mut triples, checked_len, reservation) {
                    // On a denied scratch reservation, preserve condition precedence.
                    // This slow path is reached only when admission cannot reserve storage.
                    for (instance_index, pair) in
                        instances.iter().take(checked_len).enumerate().skip(1)
                    {
                        if instances
                            .iter()
                            .take(instance_index)
                            .any(|prior| prior == pair)
                        {
                            return Err(ExtentRefusal::DuplicateTriple {
                                relationship,
                                extent_index,
                                instance_index,
                            });
                        }
                    }
                    if let Some((_, mismatch)) = first_mismatch {
                        return Err(mismatch);
                    }
                    return Err(extent_storage(storage));
                }
                triples.extend(
                    instances
                        .iter()
                        .take(checked_len)
                        .enumerate()
                        .map(|(index, (source, target))| (source, target, index)),
                );
                triples.sort_unstable_by(|a, b| (a.0, a.1, a.2).cmp(&(b.0, b.1, b.2)));
                let first_duplicate = triples
                    .windows(2)
                    .filter_map(|pair| {
                        (pair[0].0 == pair[1].0 && pair[0].1 == pair[1].1).then_some(pair[1].2)
                    })
                    .min();
                if let Some(instance_index) = first_duplicate {
                    return Err(ExtentRefusal::DuplicateTriple {
                        relationship,
                        extent_index,
                        instance_index,
                    });
                }
            }
            if let Some((_, mismatch)) = first_mismatch {
                return Err(mismatch);
            }
            let mut links = Vec::new();
            reserve_elements(&mut links, instances.len(), reservation).map_err(extent_storage)?;
            links.extend(instances);
            links.sort_unstable();
            let mut inverse = Vec::new();
            reserve_elements(&mut inverse, links.len(), reservation).map_err(extent_storage)?;
            inverse.extend(
                links
                    .iter()
                    .map(|(source, target)| (target.clone(), source.clone())),
            );
            inverse.sort_unstable();
            admitted
                .try_insert(
                    relationship,
                    StoredExtent {
                        complete,
                        links,
                        inverse,
                    },
                    reservation,
                )
                .map_err(extent_storage)?;
        }
        Ok(Self { extents: admitted })
    }

    /// The listed state of `relationship`.
    pub fn state(&self, relationship: &K) -> ExtentState {
        match self.extents.get(relationship) {
            None => ExtentState::Unlisted,
            Some(extent) if extent.complete => ExtentState::Complete,
            Some(_) => ExtentState::Incomplete,
        }
    }

    /// Whether this relationship has a listed complete extent.
    pub fn complete(&self, relationship: &K) -> bool {
        self.state(relationship) == ExtentState::Complete
    }

    /// Targets for the whole source reference, ordered by whole target key.
    pub fn targets<'a>(
        &'a self,
        relationship: &K,
        source: &ObjectReference,
    ) -> impl Iterator<Item = &'a ObjectReference> {
        let links = self
            .extents
            .get(relationship)
            .map_or(&[][..], |extent| extent.links.as_slice());
        let start = links.partition_point(|(candidate, _)| candidate < source);
        let end = links.partition_point(|(candidate, _)| candidate <= source);
        links
            .get(start..end)
            .into_iter()
            .flatten()
            .map(|(_, target)| target)
    }

    /// Sources for the whole target reference, ordered by whole source key.
    pub fn sources<'a>(
        &'a self,
        relationship: &K,
        target: &ObjectReference,
    ) -> impl Iterator<Item = &'a ObjectReference> {
        let inverse = self
            .extents
            .get(relationship)
            .map_or(&[][..], |extent| extent.inverse.as_slice());
        let start = inverse.partition_point(|(candidate, _)| candidate < target);
        let end = inverse.partition_point(|(candidate, _)| candidate <= target);
        inverse
            .get(start..end)
            .into_iter()
            .flatten()
            .map(|(_, source)| source)
    }
}

fn extent_storage<K>(failure: StorageFailure) -> ExtentRefusal<K> {
    match failure {
        StorageFailure::Allocation(request) => ExtentRefusal::Allocation(request),
        StorageFailure::Capacity(request) => ExtentRefusal::Capacity(request),
    }
}

/// A checked population row with caller ordered, nonempty covered types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopulationEntry<P: Ord> {
    population: P,
    covered_types: Vec<EffectiveId>,
    complete: bool,
}

/// Why a population entry could not be constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PopulationEntryFailure {
    /// No covered type was supplied.
    #[error("population entry has no covered types")]
    EmptyCoveredTypes,
    /// A covered type was repeated.
    #[error("duplicate covered type {0:?}")]
    DuplicateCoveredType(EffectiveId),
    /// A representable reservation was denied.
    #[error("population entry allocation denied: {0:?}")]
    Allocation(StorageRequest),
    /// Reservation size or representation overflowed.
    #[error("population entry capacity exceeded: {0:?}")]
    Capacity(StorageRequest),
}

impl<P: Ord> PopulationEntry<P> {
    /// Fallibly retain a nonempty, duplicate free sequence in caller order.
    pub fn try_new(
        population: P,
        covered_types: impl IntoIterator<Item = EffectiveId>,
        complete: bool,
    ) -> Result<Self, PopulationEntryFailure> {
        Self::with_reservation(population, covered_types, complete, &GlobalReservations)
    }

    fn with_reservation<R: ReservationPolicy>(
        population: P,
        covered_types: impl IntoIterator<Item = EffectiveId>,
        complete: bool,
        reservation: &R,
    ) -> Result<Self, PopulationEntryFailure> {
        let mut covered = Vec::new();
        for object_type in covered_types {
            if covered.contains(&object_type) {
                return Err(PopulationEntryFailure::DuplicateCoveredType(object_type));
            }
            reserve_elements(&mut covered, 1, reservation).map_err(entry_storage)?;
            covered.push(object_type);
        }
        if covered.is_empty() {
            return Err(PopulationEntryFailure::EmptyCoveredTypes);
        }
        Ok(Self {
            population,
            covered_types: covered,
            complete,
        })
    }

    /// The caller supplied population identity.
    pub fn population(&self) -> &P {
        &self.population
    }

    /// Covered types in their caller supplied order.
    pub fn covered_types(&self) -> &[EffectiveId] {
        &self.covered_types
    }

    /// The explicit completeness state of this population.
    pub fn complete(&self) -> bool {
        self.complete
    }
}

fn entry_storage(failure: StorageFailure) -> PopulationEntryFailure {
    match failure {
        StorageFailure::Allocation(request) => PopulationEntryFailure::Allocation(request),
        StorageFailure::Capacity(request) => PopulationEntryFailure::Capacity(request),
    }
}

/// Pulls authenticated population rows from the model owner.
pub trait PopulationEnumerator<P: Ord + Clone> {
    /// The enumerator's original failure type.
    type Error;
    /// `None` means normal exhaustion; an error means incomplete traversal.
    fn next_row(&mut self) -> Result<Option<PopulationEntry<P>>, Self::Error>;
}

/// Why retaining a population row failed.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ManifestFailure<P> {
    /// A retained population identity was repeated.
    #[error("duplicate population")]
    DuplicatePopulation(P),
    /// A retained row names a type absent from the environment.
    #[error("unknown covered type")]
    UnknownCoveredType {
        /// Population identity.
        population: P,
        /// First unknown type in caller order.
        covered_type: EffectiveId,
    },
    /// A representable reservation was denied.
    #[error("manifest allocation denied: {0:?}")]
    Allocation(StorageRequest),
    /// Reservation size or representation overflowed.
    #[error("manifest capacity exceeded: {0:?}")]
    Capacity(StorageRequest),
}

/// Why population enumeration stopped before normal exhaustion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StopReason<E> {
    /// One probe row exceeded the number retained.
    RowLimit {
        /// Maximum number of retained rows.
        limit: usize,
    },
    /// The enumerator failed, retaining the rows before this pull.
    EnumeratorError(E),
}

/// Manifest traversal state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestState {
    /// Normal exhaustion established closure.
    Closed,
    /// Enumeration stopped before exhaustion.
    Partial,
}

/// Population rows proved complete by normal enumerator exhaustion.
#[derive(Clone, Debug)]
pub struct ClosedPopulationManifest<'t, P: Ord + Clone> {
    rows: Vec<PopulationEntry<P>>,
    coverage: Vec<(EffectiveId, bool)>,
    source_rows: usize,
    types: &'t TypeEnvironment,
}

/// Rows retained before a row limit or enumerator error.
#[derive(Clone, Debug)]
pub struct PartialPopulationRows<'t, P: Ord + Clone, E> {
    rows: Vec<PopulationEntry<P>>,
    source_rows: usize,
    stop: StopReason<E>,
    types: &'t TypeEnvironment,
}

/// Closed or partial population enumeration, with no caller controlled state constructor.
#[derive(Clone, Debug)]
pub struct ManifestOutcome<'t, P: Ord + Clone, E> {
    state: OutcomeState<'t, P, E>,
}

#[derive(Clone, Debug)]
enum OutcomeState<'t, P: Ord + Clone, E> {
    Closed(ClosedPopulationManifest<'t, P>),
    Partial(PartialPopulationRows<'t, P, E>),
}

impl<'t, P: Ord + Clone> ClosedPopulationManifest<'t, P> {
    /// Enumerate at most `row_limit` retained rows and one probe row.
    pub fn from_enumerator<E: PopulationEnumerator<P>>(
        types: &'t TypeEnvironment,
        enumerator: &mut E,
        row_limit: usize,
    ) -> Result<ManifestOutcome<'t, P, E::Error>, ManifestFailure<P>> {
        Self::with_reservation(types, enumerator, row_limit, &GlobalReservations)
    }

    fn with_reservation<E: PopulationEnumerator<P>, R: ReservationPolicy>(
        types: &'t TypeEnvironment,
        enumerator: &mut E,
        row_limit: usize,
        reservation: &R,
    ) -> Result<ManifestOutcome<'t, P, E::Error>, ManifestFailure<P>> {
        let mut rows: Vec<PopulationEntry<P>> = Vec::new();
        let mut source_rows = 0usize;
        loop {
            match enumerator.next_row() {
                Ok(None) => {
                    let coverage = build_coverage(types, &rows, reservation)?;
                    return Ok(ManifestOutcome {
                        state: OutcomeState::Closed(Self {
                            rows,
                            coverage,
                            source_rows,
                            types,
                        }),
                    });
                }
                Err(error) => {
                    return Ok(ManifestOutcome {
                        state: OutcomeState::Partial(PartialPopulationRows {
                            rows,
                            source_rows,
                            stop: StopReason::EnumeratorError(error),
                            types,
                        }),
                    })
                }
                Ok(Some(row)) => {
                    source_rows = source_rows.checked_add(1).ok_or_else(|| {
                        ManifestFailure::Capacity(StorageRequest::new(
                            1,
                            StorageUnit::AdditionalElements,
                        ))
                    })?;
                    if rows.len() == row_limit {
                        return Ok(ManifestOutcome {
                            state: OutcomeState::Partial(PartialPopulationRows {
                                rows,
                                source_rows,
                                stop: StopReason::RowLimit { limit: row_limit },
                                types,
                            }),
                        });
                    }
                    let position = match rows
                        .binary_search_by(|held| held.population.cmp(&row.population))
                    {
                        Ok(_) => return Err(ManifestFailure::DuplicatePopulation(row.population)),
                        Err(position) => position,
                    };
                    for covered_type in &row.covered_types {
                        if types.object_type(*covered_type).is_none() {
                            return Err(ManifestFailure::UnknownCoveredType {
                                population: row.population,
                                covered_type: *covered_type,
                            });
                        }
                    }
                    reserve_elements(&mut rows, 1, reservation).map_err(manifest_storage)?;
                    rows.insert(position, row);
                }
            }
        }
    }
}

fn manifest_storage<P>(failure: StorageFailure) -> ManifestFailure<P> {
    match failure {
        StorageFailure::Allocation(request) => ManifestFailure::Allocation(request),
        StorageFailure::Capacity(request) => ManifestFailure::Capacity(request),
    }
}

fn build_coverage<P: Ord, R: ReservationPolicy>(
    types: &TypeEnvironment,
    rows: &[PopulationEntry<P>],
    reservation: &R,
) -> Result<Vec<(EffectiveId, bool)>, ManifestFailure<P>> {
    let mut coverage = Vec::new();
    for declared in types.object_types() {
        let actual = declared.key();
        let mut any = false;
        let mut all_complete = true;
        for row in rows {
            if row
                .covered_types
                .iter()
                .any(|member| types.conforms(actual, *member))
            {
                any = true;
                all_complete &= row.complete;
            }
        }
        if any {
            reserve_elements(&mut coverage, 1, reservation).map_err(manifest_storage)?;
            coverage.push((actual, all_complete));
        }
    }
    coverage.sort_unstable_by_key(|(object_type, _)| *object_type);
    Ok(coverage)
}

impl<'t, P: Ord + Clone> ClosedPopulationManifest<'t, P> {
    /// Retained rows sorted by population identity.
    pub fn rows(&self) -> impl Iterator<Item = &PopulationEntry<P>> {
        self.rows.iter()
    }
    /// Number of pulled source rows.
    pub fn source_rows(&self) -> usize {
        self.source_rows
    }
}

impl<'t, P: Ord + Clone, E> PartialPopulationRows<'t, P, E> {
    /// Retained rows sorted by population identity.
    pub fn rows(&self) -> impl Iterator<Item = &PopulationEntry<P>> {
        self.rows.iter()
    }
    /// Number of pulled source rows, including a row limit probe.
    pub fn source_rows(&self) -> usize {
        self.source_rows
    }
    /// The typed stop reason.
    pub fn stop(&self) -> &StopReason<E> {
        &self.stop
    }
}

impl<'t, P: Ord + Clone, E> ManifestOutcome<'t, P, E> {
    /// Whether enumeration exhausted normally.
    pub fn state(&self) -> ManifestState {
        match self.state {
            OutcomeState::Closed(_) => ManifestState::Closed,
            OutcomeState::Partial(_) => ManifestState::Partial,
        }
    }
    /// The closed view, only after normal exhaustion.
    pub fn closed(&self) -> Option<&ClosedPopulationManifest<'t, P>> {
        match &self.state {
            OutcomeState::Closed(closed) => Some(closed),
            OutcomeState::Partial(_) => None,
        }
    }
    /// The partial view, only after an early stop.
    pub fn partial(&self) -> Option<&PartialPopulationRows<'t, P, E>> {
        match &self.state {
            OutcomeState::Closed(_) => None,
            OutcomeState::Partial(partial) => Some(partial),
        }
    }
    /// Retained rows sorted by population identity.
    pub fn rows(&self) -> impl Iterator<Item = &PopulationEntry<P>> {
        match &self.state {
            OutcomeState::Closed(closed) => closed.rows.iter(),
            OutcomeState::Partial(partial) => partial.rows.iter(),
        }
    }
    /// Number of successfully pulled rows, including a row limit probe.
    pub fn source_rows(&self) -> usize {
        match &self.state {
            OutcomeState::Closed(closed) => closed.source_rows,
            OutcomeState::Partial(partial) => partial.source_rows,
        }
    }
    /// The stop reason, absent only after normal exhaustion.
    pub fn stop(&self) -> Option<&StopReason<E>> {
        self.partial().map(PartialPopulationRows::stop)
    }

    pub(crate) fn closed_coverage(&self, types: &TypeEnvironment, actual: EffectiveId) -> bool {
        let OutcomeState::Closed(closed) = &self.state else {
            return false;
        };
        if !ptr::eq(closed.types, types) || types.object_type(actual).is_none() {
            return false;
        }
        closed
            .coverage
            .binary_search_by_key(&actual, |(object_type, _)| *object_type)
            .ok()
            .and_then(|position| closed.coverage.get(position))
            .is_some_and(|(_, complete)| *complete)
    }
}

impl<'t, P: Ord + Clone + PartialEq> PartialEq for ClosedPopulationManifest<'t, P> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.types, other.types)
            && self.source_rows == other.source_rows
            && self.rows == other.rows
    }
}
impl<'t, P: Ord + Clone + Eq> Eq for ClosedPopulationManifest<'t, P> {}
impl<'t, P: Ord + Clone + PartialEq, E: PartialEq> PartialEq for PartialPopulationRows<'t, P, E> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.types, other.types)
            && self.source_rows == other.source_rows
            && self.rows == other.rows
            && self.stop == other.stop
    }
}
impl<'t, P: Ord + Clone + Eq, E: Eq> Eq for PartialPopulationRows<'t, P, E> {}
impl<'t, P: Ord + Clone + PartialEq, E: PartialEq> PartialEq for ManifestOutcome<'t, P, E> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.state, &other.state) {
            (OutcomeState::Closed(a), OutcomeState::Closed(b)) => a == b,
            (OutcomeState::Partial(a), OutcomeState::Partial(b)) => a == b,
            (OutcomeState::Closed(_), OutcomeState::Partial(_))
            | (OutcomeState::Partial(_), OutcomeState::Closed(_)) => false,
        }
    }
}
impl<'t, P: Ord + Clone + Eq, E: Eq> Eq for ManifestOutcome<'t, P, E> {}

/// Result of resolving a full object reference against held objects and population evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Resolution {
    /// The object is admitted in the closure.
    Present,
    /// Exhaustive, complete covering evidence proves the object missing.
    Dangling,
    /// The supplied evidence cannot establish absence.
    Undecidable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declaration::ObjectTypeDeclaration;
    use alloc::vec;
    use core::cell::Cell;
    use ix_trace_rs::trace;
    use quire_exact::{ObjectId, UniverseId};

    struct CapacityPolicy;

    impl ReservationPolicy for CapacityPolicy {
        fn permit(&self, _request: StorageRequest) -> bool {
            true
        }
        fn check_request(&self, request: StorageRequest) -> Result<(), StorageFailure> {
            Err(StorageFailure::Capacity(request))
        }
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "fixture supplies identities; production mints none"
    )]
    fn ty(byte: u8) -> EffectiveId {
        EffectiveId::from_digest([byte; 32])
    }

    fn types() -> TypeEnvironment {
        TypeEnvironment::new(
            [],
            [
                ObjectTypeDeclaration::new(ty(1), "A", vec![]),
                ObjectTypeDeclaration::new(ty(2), "B", vec![]),
            ],
        )
        .expect("fixture model")
    }

    fn reference(object_type: u8) -> ObjectReference {
        ObjectReference::new(
            UniverseId::from_digest([9; 32]),
            ty(object_type),
            ObjectId::new("x").expect("fixture id"),
        )
    }

    fn declaration() -> NavigationRelationship<&'static str> {
        NavigationRelationship {
            key: "r",
            source: RelationshipEnd {
                object_type: ty(1),
                unique: true,
            },
            target: RelationshipEnd {
                object_type: ty(2),
                unique: true,
            },
        }
    }

    #[trace("TC-919", "FR-119-AC-11")]
    #[test]
    fn table_and_extent_reservations_refuse_exact_request_then_retry() {
        let types = types();
        let observed = Cell::new(None);
        let deny = |request| {
            observed.set(Some(request));
            false
        };
        let request = StorageRequest::new(1, StorageUnit::AdditionalElements);
        assert_eq!(
            NavigationRelationshipTable::with_reservation(&types, [declaration()], &deny),
            Err(TableRefusal::Allocation(request))
        );
        assert_eq!(observed.get(), Some(request));
        let table = NavigationRelationshipTable::new(&types, [declaration()]).unwrap();
        let before = table.clone();
        let extent = || RelationshipExtent {
            relationship: "r",
            complete: true,
            instances: vec![(reference(1), reference(2))],
        };
        let prior_store = RelationshipExtentStore::new(&types, &table, [extent()]).unwrap();
        let prior_before = prior_store.clone();
        observed.set(None);
        assert_eq!(
            RelationshipExtentStore::with_reservation(&types, &table, [extent()], &deny),
            Err(ExtentRefusal::Allocation(request))
        );
        assert_eq!(observed.get(), Some(request));
        assert_eq!(prior_store, prior_before);
        let repeated = RelationshipExtent {
            relationship: "r",
            complete: true,
            instances: vec![(reference(1), reference(2)), (reference(1), reference(2))],
        };
        assert_eq!(
            RelationshipExtentStore::with_reservation(&types, &table, [repeated], &deny),
            Err(ExtentRefusal::DuplicateTriple {
                relationship: "r",
                extent_index: 0,
                instance_index: 1
            })
        );
        let store = RelationshipExtentStore::new(&types, &table, [extent()]).unwrap();
        let store_before = store.clone();
        assert!(store.complete(&"r"));
        assert_eq!(store, store_before);
        assert_eq!(table, before);
        assert_eq!(
            NavigationRelationshipTable::with_reservation(&types, [declaration()], &CapacityPolicy),
            Err(TableRefusal::Capacity(request))
        );
        assert_eq!(
            RelationshipExtentStore::with_reservation(&types, &table, [extent()], &CapacityPolicy),
            Err(ExtentRefusal::Capacity(request))
        );
        let overflow = reserve_elements::<u64, _>(&mut Vec::new(), usize::MAX, &GlobalReservations)
            .expect_err("array layout cannot represent max elements");
        assert_eq!(
            table_storage::<&str>(overflow),
            TableRefusal::Capacity(StorageRequest::new(
                usize::MAX,
                StorageUnit::AdditionalElements
            ))
        );
        assert_eq!(
            extent_storage::<&str>(overflow),
            ExtentRefusal::Capacity(StorageRequest::new(
                usize::MAX,
                StorageUnit::AdditionalElements
            ))
        );
    }

    struct OneRow(Option<PopulationEntry<&'static str>>);
    struct TwoRows(alloc::vec::IntoIter<PopulationEntry<&'static str>>);
    impl PopulationEnumerator<&'static str> for TwoRows {
        type Error = ();
        fn next_row(&mut self) -> Result<Option<PopulationEntry<&'static str>>, Self::Error> {
            Ok(self.0.next())
        }
    }
    impl PopulationEnumerator<&'static str> for OneRow {
        type Error = ();
        fn next_row(&mut self) -> Result<Option<PopulationEntry<&'static str>>, Self::Error> {
            Ok(self.0.take())
        }
    }

    #[trace("TC-919", "FR-119-AC-15")]
    #[test]
    fn entry_and_manifest_storage_refuse_exact_request_and_capacity() {
        let request = StorageRequest::new(1, StorageUnit::AdditionalElements);
        let observed = Cell::new(None);
        let deny = |request| {
            observed.set(Some(request));
            false
        };
        assert_eq!(
            PopulationEntry::with_reservation("Q", [ty(1)], true, &deny),
            Err(PopulationEntryFailure::Allocation(request))
        );
        assert_eq!(observed.get(), Some(request));
        let types = types();
        let make_row = || PopulationEntry::try_new("Q", [ty(1)], true).unwrap();
        let mut source = OneRow(Some(make_row()));
        observed.set(None);
        assert_eq!(
            ClosedPopulationManifest::with_reservation(&types, &mut source, 1, &deny),
            Err(ManifestFailure::Allocation(request))
        );
        assert_eq!(observed.get(), Some(request));
        let mut retry = OneRow(Some(make_row()));
        let admitted = ClosedPopulationManifest::from_enumerator(&types, &mut retry, 1).unwrap();
        assert_eq!(admitted.state(), ManifestState::Closed);
        let mut duplicate_source = TwoRows(vec![make_row(), make_row()].into_iter());
        let reservations = Cell::new(0);
        let deny_second = |_: StorageRequest| {
            reservations.set(reservations.get() + 1);
            reservations.get() == 1
        };
        assert_eq!(
            ClosedPopulationManifest::with_reservation(
                &types,
                &mut duplicate_source,
                2,
                &deny_second
            ),
            Err(ManifestFailure::DuplicatePopulation("Q"))
        );
        assert_eq!(reservations.get(), 1);
        assert_eq!(
            PopulationEntry::with_reservation("Q", [ty(1)], true, &CapacityPolicy),
            Err(PopulationEntryFailure::Capacity(request))
        );
        let mut capacity_source = OneRow(Some(make_row()));
        assert_eq!(
            ClosedPopulationManifest::with_reservation(
                &types,
                &mut capacity_source,
                1,
                &CapacityPolicy
            ),
            Err(ManifestFailure::Capacity(request))
        );
        let overflow = reserve_elements::<u64, _>(&mut Vec::new(), usize::MAX, &GlobalReservations)
            .expect_err("array layout cannot represent max elements");
        assert_eq!(
            entry_storage(overflow),
            PopulationEntryFailure::Capacity(StorageRequest::new(
                usize::MAX,
                StorageUnit::AdditionalElements
            ))
        );
        assert_eq!(
            manifest_storage::<&str>(overflow),
            ManifestFailure::Capacity(StorageRequest::new(
                usize::MAX,
                StorageUnit::AdditionalElements
            ))
        );
    }
}
