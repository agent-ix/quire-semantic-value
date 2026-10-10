// SPDX-License-Identifier: AGPL-3.0-or-later
//! The QSpec FR-143 declared record, tuple and model object-type registry, and
//! the QSpec FR-149 checked equality layer over it.
//!
//! It owns the registry (`TypeEnvironment`, `ObjectTypeDeclaration`,
//! `CompositeDeclaration`, `CompositeShape`, `InvalidDeclaration`,
//! `RecursionEdges`, `DeclarationCause`) and the check-level equality layer
//! (`EqualityOperator`, `EqualityOperand`, `EqualitySchedule`,
//! `CheckedEquality`, `TypeEnvironment::check_equality`,
//! `admits_equality_conversion`, `operand_value`). The environment also
//! carries the package's quantity [`UnitTable`], since a `ValueType::Quantity`
//! names its unit only by id. None of these are kernel types, so they live here,
//! over `quire_exact`'s own `Value`/`ValueType`, where a compiler and a backend
//! share them. An object type's operations carry the domain package's effect
//! frame, a caller's model type, so they stay with the caller beside the
//! environment, and a reached [`TypeEnvironmentLimits`] ceiling is this
//! module's own [`EnvironmentLimit`], which a compiler stage maps to its
//! stage limit.
//!
//! The equality layer lives here because it is parameterized over a
//! `TypeEnvironment` and a checked `ValueType`. The occurrence-pair walk it
//! schedules is [`quire_exact::planned_equality`]/[`quire_exact::plan_equality`],
//! called directly by `CheckedEquality::run`: the kernel's own `plan_pairs` is
//! `pub(crate)` there, and this layer never needs the lower-level pair count that
//! an evaluator gets straight from `quire_exact::member_equal`.
//!
//! [`FieldDeclaration`], [`Component`], [`ConstructionCause`] and
//! [`ConstructionRefusal`] are not kernel types: a field is identified here by
//! its declared *name* (a `str`-keyed lookup, `match_names`/`fill_slots`), not by
//! the kernel's opaque `MemberId` (`quire_exact::FieldDeclaration`'s own key).
//! `ConstructionCause` also carries `UnknownDeclaration`, which the kernel's own
//! `ConstructionCause` has no need of: the kernel's `record`/`tuple` take their
//! declared shape directly, with no registry lookup to fail, while this module's
//! `TypeEnvironment::record`/`tuple`/`evaluate_record`/`evaluate_tuple` look a
//! `NodeKey` up in the registry first and must report that lookup's own failure.
//! Because of this, `TypeEnvironment` cannot call the kernel's checked
//! `record`/`tuple` constructors either (they need the kernel's `MemberId`-keyed
//! `FieldDeclaration`); it does its own name-keyed checking through
//! `fill_slots`/`match_names`, and then calls the kernel's trusted, unchecked
//! [`from_admitted_slots`] to materialize the result -- mirroring
//! `quire_exact::OptionValue::from_admitted`'s identical bypass role.
//!
//! QSL FR-089-AC-6: kernel `ValueType::admits` refuses every
//! `(ValueType::Population, Value::Population)` pair outright -- the
//! declared-maximum comparison (QSL FR-089-AC-5) is the caller layer's own check.
//! `Population` is QSpec FR-153's own restriction: it is never nested inside a
//! record field, tuple position, option payload or collection element (every
//! such context is refused earlier, at declaration admission, by
//! `TypeEnvironment::type_refusal`), so none of this module's `admits()`
//! calls (`fill_slots`'s field check) ever receive a `Population` pair; only an
//! evaluator's top-level parameter-admission loop needs the QSL FR-089-AC-5
//! compensation, since `Population<T>[N]` is reachable there directly as a bare
//! parameter type.

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use quire_exact::{
    compare_shifted, power_of_ten_bits, sbits, sdigits, Charge, ChargePoint, CheckedInvariantCause,
    ComparisonOperator, Decimal, DecimalOperation, DecimalType, IllTyped, IllTypedCause, Integer,
    LimitKind, Meter, Outcome, Presence, Quantity, Rational, Refusal,
};
use quire_exact::{from_admitted_slots, retain_composite, Deferred, FieldValue, Value, ValueType};

use crate::enumeration::{compare_enum, EnumMemberIndex};
use crate::quantity::{
    compare_quantity, convert_quantity, ConvertedValue, QuantityTarget, UnitScope, UnitTable,
};
use crate::stop::{outcome_from_stop, outcome_into_stop, Stop};
use core::alloc::Layout;
use core::fmt;
use quire_exact::CollectionKind;
use quire_exact::EffectiveId;
use quire_exact::EnumShape;
use quire_exact::NodeKey;
use quire_exact::{compare_text, evaluate_decimal};

/// One object-type field's identity: the object type that declares it and
/// its declared name (QSpec FR-151 field redefinition names its target this way).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FieldRef {
    /// The declaring object type's effective identity.
    pub owner: EffectiveId,
    /// The field's declared name.
    pub name: String,
}

#[cfg(test)]
mod storage_reservation_tests {
    use super::*;
    use core::cell::Cell;
    use ix_trace_rs::trace;

    struct DenyFirst {
        seen: Cell<Option<StorageRequest>>,
    }

    impl ReservationPolicy for DenyFirst {
        fn permit(&self, request: StorageRequest) -> bool {
            self.seen.set(Some(request));
            false
        }
    }

    /// Trace: FR-108-AC-1, FR-108-AC-3
    #[trace("TC-907", "FR-108-AC-1", "FR-108-AC-3")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies a checked declaration key without minting one in production"
    )]
    #[test]
    fn registry_denial_retains_reservation_and_retry_admits() {
        let declarations = || {
            [CompositeDeclaration::new(
                NodeKey::from_digest([1; 32]),
                "R",
                CompositeShape::Tuple(Vec::new()),
            )]
        };
        let deny = DenyFirst {
            seen: Cell::new(None),
        };
        let denied = TypeEnvironment::bounded_with_reservations(
            declarations(),
            [],
            TypeEnvironmentLimits::default(),
            None,
            &deny,
        );
        let request = deny.seen.get().expect("registry made a reservation");
        assert_eq!(denied, Err(EnvironmentFailure::Allocation(request)));
        assert_eq!(
            request,
            StorageRequest::new(1, StorageUnit::AdditionalElements)
        );
        let admitted = TypeEnvironment::new(declarations(), []).expect("retry admits");
        assert_eq!(admitted.composites().count(), 1);
    }

    /// Trace: FR-108-AC-2
    #[trace("TC-907", "FR-108-AC-2")]
    #[test]
    fn size_overflow_is_distinct_from_allocator_denial() {
        let mut values: Vec<u8> = Vec::new();
        assert_eq!(
            reserve_elements(&mut values, usize::MAX, &GlobalReservations),
            Err(EnvironmentFailure::Capacity(StorageRequest::new(
                usize::MAX,
                StorageUnit::AdditionalElements,
            )))
        );
    }

    struct DenyBytes {
        seen: Cell<Option<StorageRequest>>,
    }

    impl ReservationPolicy for DenyBytes {
        fn permit(&self, request: StorageRequest) -> bool {
            if request.unit() == StorageUnit::Bytes {
                self.seen.set(Some(request));
                false
            } else {
                true
            }
        }
    }

    /// Trace: FR-108-AC-1, FR-108-AC-2, FR-108-AC-4
    #[trace("TC-907", "FR-108-AC-1", "FR-108-AC-2", "FR-108-AC-4")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies checked declaration keys without minting them in production"
    )]
    #[test]
    fn admission_byte_denial_preserves_request_and_invalid_declaration_classification() {
        let declarations = || {
            [
                CompositeDeclaration::new(
                    NodeKey::from_digest([1; 32]),
                    "first",
                    CompositeShape::Tuple(vec![]),
                ),
                CompositeDeclaration::new(
                    NodeKey::from_digest([1; 32]),
                    "second",
                    CompositeShape::Tuple(vec![]),
                ),
            ]
        };
        let deny = DenyBytes {
            seen: Cell::new(None),
        };
        assert_eq!(
            TypeEnvironment::bounded_with_reservations(
                declarations(),
                [],
                TypeEnvironmentLimits::default(),
                None,
                &deny,
            ),
            Err(EnvironmentFailure::Allocation(StorageRequest::new(
                6,
                StorageUnit::Bytes,
            )))
        );
        assert_eq!(
            deny.seen.get(),
            Some(StorageRequest::new(6, StorageUnit::Bytes))
        );
        assert_eq!(
            TypeEnvironment::new(declarations(), []),
            Err(EnvironmentFailure::Refused(InvalidDeclaration {
                declaration: "second".into(),
                cause: DeclarationCause::DuplicateKey,
            }))
        );
    }

    /// Trace: FR-108-AC-4
    #[trace("TC-907", "FR-108-AC-4")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies a checked identity without minting it in production"
    )]
    #[test]
    fn cancellation_keeps_the_work_limit_classification() {
        let cancel = quire_exact::Cancel::new();
        cancel.cancel(quire_exact::CancelCause::Requested);
        let denied = TypeEnvironment::bounded_with_cancel(
            [],
            [ObjectTypeDeclaration::new(
                EffectiveId::from_digest([8; 32]),
                "O",
                vec![FieldDeclaration::new(
                    "field",
                    ValueType::Boolean,
                    Presence::Required,
                )],
            )],
            TypeEnvironmentLimits::default(),
            &cancel,
        );
        let Err(EnvironmentFailure::Limit(limit)) = denied else {
            panic!("cancelled admission must keep the work-limit classification");
        };
        assert_eq!(limit.kind(), EnvironmentLimitKind::WorkUnits);
        assert_eq!(limit.configured_bound(), DEFAULT_WORK_UNITS);
        assert_eq!(limit.actual(), 1);
        assert_eq!(cancel.tripped(), Some(quire_exact::CancelCause::Requested));
    }

    /// Trace: FR-108-AC-4
    #[trace("TC-907", "FR-108-AC-4")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies a checked type identity without minting one in production"
    )]
    #[test]
    fn admission_work_units_admits_at_n_and_reports_n_at_n_minus_one() {
        let key = EffectiveId::from_digest([9; 32]);
        let declarations = || {
            [ObjectTypeDeclaration::new(
                key,
                "O",
                vec![FieldDeclaration::new(
                    "field",
                    ValueType::Boolean,
                    Presence::Required,
                )],
            )]
        };
        let limits = |work_units| TypeEnvironmentLimits {
            work_units,
            ..TypeEnvironmentLimits::default()
        };
        let admitted = TypeEnvironment::bounded([], declarations(), limits(2))
            .expect("one field costs exactly two admission work units");
        assert!(admitted.attribute(key, "field").is_some());
        let Err(EnvironmentFailure::Limit(limit)) =
            TypeEnvironment::bounded([], declarations(), limits(1))
        else {
            panic!("one fewer work unit must stop public admission");
        };
        assert_eq!(limit.kind(), EnvironmentLimitKind::WorkUnits);
        assert_eq!(limit.configured_bound(), 1);
        assert_eq!(limit.actual(), 2);
    }

    /// Trace: FR-108-AC-3, FR-108-AC-5
    #[trace("TC-907", "FR-108-AC-3", "FR-108-AC-5")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies checked type identities without minting them in production"
    )]
    #[test]
    fn inherited_attribute_view_resolves_the_original_field() {
        let a = EffectiveId::from_digest([1; 32]);
        let b = EffectiveId::from_digest([2; 32]);
        let parent = ObjectTypeDeclaration::new(
            a,
            "A",
            vec![FieldDeclaration::new(
                "x",
                ValueType::Integer,
                Presence::Required,
            )],
        );
        let child = ObjectTypeDeclaration::new(b, "B", vec![]).with_supertypes(vec![a]);
        let environment = TypeEnvironment::new([], [child, parent]).expect("inheritance admits");
        let attributes = environment.attributes(b).expect("child has admitted type");
        assert_eq!(attributes.len(), 1);
        let inherited = attributes.iter().next().expect("one inherited attribute");
        assert_eq!(inherited.owner(), a);
        assert_eq!(inherited.field().name(), "x");
        assert!(inherited.stands_for(&FieldRef::new(a, "x")));
        assert_eq!(
            environment
                .attribute(b, "x")
                .expect("name lookup")
                .identity(),
            FieldRef::new(a, "x")
        );
    }

    /// Trace: FR-108-AC-3, FR-108-AC-5
    #[trace("TC-907", "FR-108-AC-3", "FR-108-AC-5")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies checked declaration identities without minting them in production"
    )]
    #[test]
    fn registry_order_and_inherited_slot_order_survive_reversed_inputs() {
        let record = NodeKey::from_digest([1; 32]);
        let tuple = NodeKey::from_digest([2; 32]);
        let parent = EffectiveId::from_digest([1; 32]);
        let child = EffectiveId::from_digest([2; 32]);
        let environment = TypeEnvironment::new(
            [
                CompositeDeclaration::new(tuple, "T", CompositeShape::Tuple(vec![])),
                CompositeDeclaration::new(record, "R", CompositeShape::Record(vec![])),
            ],
            [
                ObjectTypeDeclaration::new(
                    child,
                    "Child",
                    vec![FieldDeclaration::new(
                        "own",
                        ValueType::Integer,
                        Presence::Required,
                    )],
                )
                .with_supertypes(vec![parent]),
                ObjectTypeDeclaration::new(
                    parent,
                    "Parent",
                    vec![FieldDeclaration::new(
                        "inherited",
                        ValueType::Boolean,
                        Presence::Required,
                    )],
                ),
            ],
        )
        .expect("reversed declarations admit");
        assert_eq!(
            environment
                .composites()
                .map(CompositeDeclaration::key)
                .collect::<Vec<_>>(),
            vec![record, tuple]
        );
        assert_eq!(
            environment
                .object_types()
                .map(ObjectTypeDeclaration::key)
                .collect::<Vec<_>>(),
            vec![parent, child]
        );
        let slots = environment.attributes(child).expect("child admits");
        assert_eq!(
            slots
                .iter()
                .map(|slot| (slot.owner(), slot.field().name()))
                .collect::<Vec<_>>(),
            vec![(child, "own"), (parent, "inherited")]
        );
        assert!(environment
            .attribute(child, "inherited")
            .expect("inherited link resolves")
            .stands_for(&FieldRef::new(parent, "inherited")));
    }

    struct DenyPhase {
        target: StoragePhase,
        current: Cell<Option<StoragePhase>>,
        denied: Cell<Option<StorageRequest>>,
    }

    impl ReservationPolicy for DenyPhase {
        fn phase(&self, phase: StoragePhase) {
            self.current.set(Some(phase));
        }

        fn permit(&self, request: StorageRequest) -> bool {
            if self.current.get() == Some(self.target) && self.denied.get().is_none() {
                self.denied.set(Some(request));
                false
            } else {
                true
            }
        }
    }

    /// Trace: FR-108-AC-1, FR-108-AC-3, FR-108-AC-5
    #[trace("TC-907", "FR-108-AC-1", "FR-108-AC-3", "FR-108-AC-5")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the fixture supplies checked identities without minting them in production"
    )]
    #[test]
    fn denial_at_each_admission_phase_is_atomic_and_retryable() {
        let declarations = || {
            (
                [
                    CompositeDeclaration::new(
                        NodeKey::from_digest([3; 32]),
                        "R",
                        CompositeShape::Record(vec![FieldDeclaration::new(
                            "field",
                            ValueType::Integer,
                            Presence::Required,
                        )]),
                    ),
                    CompositeDeclaration::new(
                        NodeKey::from_digest([4; 32]),
                        "T",
                        CompositeShape::Tuple(vec![ValueType::Boolean]),
                    ),
                ],
                [ObjectTypeDeclaration::new(
                    EffectiveId::from_digest([5; 32]),
                    "O",
                    vec![FieldDeclaration::new(
                        "x",
                        ValueType::Integer,
                        Presence::Required,
                    )],
                )],
            )
        };
        let (composites, objects) = declarations();
        let expected = TypeEnvironment::new(composites, objects).expect("fixture admits");
        for target in [
            StoragePhase::Registry,
            StoragePhase::MemberTypes,
            StoragePhase::Traversal,
            StoragePhase::Ancestors,
            StoragePhase::FieldIndex,
            StoragePhase::Sealing,
            StoragePhase::KeyJoin,
        ] {
            let policy = DenyPhase {
                target,
                current: Cell::new(None),
                denied: Cell::new(None),
            };
            let (composites, objects) = declarations();
            let denied = TypeEnvironment::bounded_with_reservations(
                composites,
                objects,
                TypeEnvironmentLimits::default(),
                None,
                &policy,
            );
            let request = policy.denied.get().expect("phase reached a reservation");
            assert_eq!(denied, Err(EnvironmentFailure::Allocation(request)));
            let (composites, objects) = declarations();
            let retried = TypeEnvironment::new(composites, objects).expect("retry admits");
            assert_eq!(retried, expected);
            let field = FieldRef::new(EffectiveId::from_digest([5; 32]), "x");
            assert!(retried
                .attribute(field.owner, &field.name)
                .expect("retry restores the member link")
                .stands_for(&field));
        }
    }
}

impl FieldRef {
    /// The field `name` declared by the object type `owner`.
    pub fn new(owner: EffectiveId, name: impl Into<String>) -> Self {
        Self {
            owner,
            name: name.into(),
        }
    }
}

/// A declaration-owned named field; its identity is (declaration key, name).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldDeclaration {
    name: String,
    value_type: ValueType,
    presence: Presence,
    /// The inherited field this one redefines (QSpec FR-151
    /// `quire.model.normalize.redefine/v1`), if any. Only an object-type
    /// field may carry one.
    redefines: Option<FieldRef>,
}

impl AsRef<FieldDeclaration> for FieldDeclaration {
    fn as_ref(&self) -> &FieldDeclaration {
        self
    }
}

impl FieldDeclaration {
    /// The field `name: value_type` or `name: value_type?`.
    pub fn new(name: impl Into<String>, value_type: ValueType, presence: Presence) -> Self {
        Self {
            name: name.into(),
            value_type,
            presence,
            redefines: None,
        }
    }

    /// This field, redefining the inherited field `target` (QSpec FR-151): the
    /// producer copies the domain package's own `redefines` link, which
    /// phase-4 normalization has already checked. In every object type that
    /// inherits both, `target` is hidden and this field takes its one slot.
    #[must_use]
    pub fn with_redefines(mut self, target: FieldRef) -> Self {
        self.redefines = Some(target);
        self
    }

    /// The inherited field this one redefines, if any.
    pub fn redefines(&self) -> Option<&FieldRef> {
        self.redefines.as_ref()
    }

    /// The field identifier.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The declared value type.
    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    /// Whether the field was declared with `?`.
    pub fn presence(&self) -> Presence {
        self.presence
    }
}

/// A record field in a record value expression. Omitting a `?` field
/// constructs `absent`.
pub enum FieldExpression<'a> {
    /// `f: e`.
    Evaluate(Deferred<'a>),
    /// `f: null`.
    Null,
}

impl fmt::Debug for FieldExpression<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Evaluate(_) => formatter.write_str("Evaluate(..)"),
            Self::Null => formatter.write_str("Null"),
        }
    }
}

/// Where a construction refusal originates.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Component {
    /// The composite value as a whole (its declaration or arity).
    Value,
    /// A named record field or object attribute.
    Field(String),
    /// A zero-based tuple position.
    Position(usize),
    /// A zero-based collection occurrence in source order.
    Element(usize),
    /// An option payload.
    Payload,
}

/// Why a construction is `refused { code: ill_typed }`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ConstructionCause {
    /// The declaration is not a record or tuple of the environment, or has
    /// the other shape.
    UnknownDeclaration,
    /// A required field is omitted.
    MissingField,
    /// A supplied field is not declared.
    UndeclaredField,
    /// A field is supplied twice.
    DuplicateField,
    /// `null` is supplied for a required field.
    NullForRequiredField,
    /// A tuple call has another argument count than its declared arity.
    WrongArity {
        /// Declared arity.
        declared: usize,
        /// Supplied arguments.
        supplied: usize,
    },
    /// A value that is not a member of the declared type.
    TypeMismatch,
}

/// A typed construction refusal at its originating component.
#[derive(Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[error("construction refused at {component:?}: {cause:?}")]
pub struct ConstructionRefusal {
    /// The originating component.
    pub component: Component,
    /// The typed cause.
    pub cause: ConstructionCause,
}

impl ConstructionRefusal {
    /// The `refused { code }` spelling.
    pub const CODE: &'static str = IllTyped::CODE;
}

/// Refuse construction of `component` with `cause`.
fn refuse<T>(component: Component, cause: ConstructionCause) -> Result<T, ConstructionRefusal> {
    Err(ConstructionRefusal { component, cause })
}

/// Index supplied entries by declared name, refusing an undeclared or
/// repeated name. This module's own `TypeEnvironment::evaluate_record` uses
/// it to match supplied `FieldExpression`s to declared fields the same way
/// [`fill_slots`] matches supplied `FieldValue`s.
fn match_names<F: AsRef<FieldDeclaration>, D: Clone + IntoIterator<Item = F>, T>(
    declared: D,
    supplied: Vec<(&str, T)>,
) -> Result<BTreeMap<&str, T>, ConstructionRefusal> {
    let mut by_name = BTreeMap::new();
    for (name, entry) in supplied {
        let component = || Component::Field(name.to_owned());
        if !declared
            .clone()
            .into_iter()
            .any(|field| field.as_ref().name == name)
        {
            return refuse(component(), ConstructionCause::UndeclaredField);
        }
        if by_name.insert(name, entry).is_some() {
            return refuse(component(), ConstructionCause::DuplicateField);
        }
    }
    Ok(by_name)
}

/// Declaration-ordered slots of a record or object from supplied fields. An
/// object's `declared` is its type's effective attribute set
/// ([`TypeEnvironment::attributes`]), so an inherited field has a slot. A
/// present value must be admitted by its field's type under `types`'
/// conformance ([`TypeEnvironment::admits`]).
pub fn fill_slots<F: AsRef<FieldDeclaration>, D: Clone + IntoIterator<Item = F>>(
    types: &TypeEnvironment,
    declared: D,
    supplied: Vec<(&str, FieldValue)>,
) -> Result<Box<[FieldValue]>, ConstructionRefusal> {
    let mut by_name = match_names(declared.clone(), supplied)?;
    let mut slots = Vec::with_capacity(declared.clone().into_iter().count());
    for field in declared {
        let field = field.as_ref();
        let component = || Component::Field(field.name.clone());
        let slot = by_name
            .remove(field.name.as_str())
            .unwrap_or(FieldValue::Absent);
        match (&slot, field.presence) {
            (FieldValue::Absent, Presence::Required) => {
                return refuse(component(), ConstructionCause::MissingField)
            }
            (FieldValue::Null, Presence::Required) => {
                return refuse(component(), ConstructionCause::NullForRequiredField)
            }
            (FieldValue::Present(value), _) if !types.admits(&field.value_type, value) => {
                return refuse(component(), ConstructionCause::TypeMismatch)
            }
            (FieldValue::Present(_) | FieldValue::Absent | FieldValue::Null, _) => {}
        }
        slots.push(slot);
    }
    Ok(slots.into_boxed_slice())
}

/// Build a composite value of `declaration` from already name-checked
/// slots, through the kernel's trusted
/// [`from_admitted_slots`](quire_exact::from_admitted_slots) bypass: this
/// module's `TypeEnvironment` construction methods have already checked
/// every slot against its own declared shape, so no second, kernel-side
/// check is needed (and the kernel's own checked `record`/`tuple` are not
/// reachable here, since they take `MemberId`-keyed declarations this
/// module does not have -- see the module doc comment).
fn composite(declaration: NodeKey, slots: Box<[FieldValue]>) -> Value {
    from_admitted_slots(declaration, slots)
}

/// The shape of a composite declaration. Complete V1 has no variant or sum
/// declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompositeShape {
    /// A record with fields in declaration order.
    Record(Vec<FieldDeclaration>),
    /// A tuple of exactly these position types.
    Tuple(Vec<ValueType>),
}

/// A record or tuple declaration with its producer-assigned node key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositeDeclaration {
    key: NodeKey,
    name: String,
    shape: CompositeShape,
}

impl CompositeDeclaration {
    /// The declaration `name` with node key `key`.
    pub fn new(key: NodeKey, name: impl Into<String>, shape: CompositeShape) -> Self {
        Self {
            key,
            name: name.into(),
            shape,
        }
    }

    /// The declaration node key.
    pub fn key(&self) -> NodeKey {
        self.key
    }

    /// The declared name, used only to name refusals.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The declared shape.
    pub fn shape(&self) -> &CompositeShape {
        &self.shape
    }
}

/// A model object type exported by a bound model, with its attributes. It is
/// keyed by its effective-declaration identity, the identity a
/// `Reference<T>` value's type component carries (QSpec FR-143), which `model`
/// computes over the domain package's effective view -- never a checked
/// node id.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectTypeDeclaration {
    key: EffectiveId,
    name: String,
    attributes: Vec<FieldDeclaration>,
    /// Every directly declared supertype (QSpec FR-151/FR-152/FR-153
    /// generalization), empty unless [`Self::with_supertypes`] sets it.
    supertypes: Vec<EffectiveId>,
}

impl ObjectTypeDeclaration {
    /// The object type `name` with declaration identity `key`, declaring no
    /// supertype. See [`Self::with_supertypes`] to declare them.
    pub fn new(
        key: EffectiveId,
        name: impl Into<String>,
        attributes: Vec<FieldDeclaration>,
    ) -> Self {
        Self {
            key,
            name: name.into(),
            attributes,
            supertypes: Vec::new(),
        }
    }

    /// Declares this object type's own direct supertypes: every key
    /// [`TypeEnvironment::new`] admits here must itself name a declared
    /// object type, and the whole supertypes graph must be acyclic. Consumes
    /// and returns `self` so every existing [`Self::new`] call site is
    /// unaffected.
    #[must_use]
    pub fn with_supertypes(mut self, supertypes: Vec<EffectiveId>) -> Self {
        self.supertypes = supertypes;
        self
    }

    /// The object type's effective-declaration identity.
    pub fn key(&self) -> EffectiveId {
        self.key
    }

    /// The declared name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The attributes this type itself declares, in declaration order. The
    /// type's full attribute set, inherited fields included, is
    /// [`TypeEnvironment::attributes`].
    pub fn attributes(&self) -> &[FieldDeclaration] {
        &self.attributes
    }

    /// Every directly declared supertype, in declaration order.
    pub fn supertypes(&self) -> &[EffectiveId] {
        &self.supertypes
    }
}

/// One attribute of an object type's effective attribute set: a field the
/// type declares or inherits and that no other field of the set redefines.
/// It is one storage slot of every object of that type.
///
/// A borrowed view of a stored attribute. The declaration is owned once by
/// the environment and is never cloned while admission flattens inheritance.
#[derive(Clone, Copy, Debug)]
pub struct EffectiveAttribute<'a> {
    environment: &'a TypeEnvironment,
    slot: &'a AttributeSlot,
    field: &'a FieldDeclaration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AttributeSlot {
    owner: EffectiveId,
    /// Position in the owner's declared attributes.
    field_index: usize,
    /// Every field this slot stands for, as positions in [`FieldTable`].
    members: Vec<usize>,
    /// This field's own position in the admission's [`FieldTable`].
    identity: usize,
}

struct EffectiveStorage {
    index: OrderedIndex<EffectiveId, Vec<usize>>,
    slots: Vec<AttributeSlot>,
}

impl<'a> EffectiveAttribute<'a> {
    /// The object type that declares this field.
    pub fn owner(&self) -> EffectiveId {
        self.slot.owner
    }

    /// The field's declaration.
    pub fn field(&self) -> &'a FieldDeclaration {
        self.field
    }

    /// This field's own identity.
    pub fn identity(&self) -> FieldRef {
        FieldRef::new(self.slot.owner, self.field.name())
    }

    /// Whether this slot holds `field`: `field` is this attribute, or a field
    /// it redefines or hides.
    pub fn stands_for(&self, field: &FieldRef) -> bool {
        self.environment
            .field_position(field)
            .is_some_and(|position| self.slot.members.binary_search(&position).is_ok())
    }
}

impl AsRef<FieldDeclaration> for EffectiveAttribute<'_> {
    fn as_ref(&self) -> &FieldDeclaration {
        self.field
    }
}

/// One object type's effective attributes in slot order.
#[derive(Clone, Copy)]
pub struct AttributeSet<'a> {
    environment: &'a TypeEnvironment,
    slots: &'a [usize],
}

impl<'a> AttributeSet<'a> {
    /// The number of storage slots.
    pub fn len(self) -> usize {
        self.slots.len()
    }

    /// Whether the object type has no attributes.
    pub fn is_empty(self) -> bool {
        self.slots.is_empty()
    }

    /// Iterate attributes in their declared slot order.
    pub fn iter(self) -> AttributeIter<'a> {
        AttributeIter {
            environment: self.environment,
            slots: self.slots.iter(),
        }
    }
}

/// Iterator over one effective attribute set.
#[derive(Clone)]
pub struct AttributeIter<'a> {
    environment: &'a TypeEnvironment,
    slots: core::slice::Iter<'a, usize>,
}

impl<'a> Iterator for AttributeIter<'a> {
    type Item = EffectiveAttribute<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.slots
            .find_map(|index| self.environment.attribute_view(*index))
    }
}

impl<'a> IntoIterator for AttributeSet<'a> {
    type Item = EffectiveAttribute<'a>;
    type IntoIter = AttributeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The QSL NFR-012 default `ancestor_steps` ceiling, in `supertypes` edges,
/// which [`TypeEnvironment::new`] admits under. It equals the model's own
/// `ModelNormalizationLimits` and `PopulationAdmissionLimits` defaults.
pub const DEFAULT_ANCESTOR_STEPS: u64 = 16_777_216;

/// The QSL NFR-012 default admission `work_units` budget, the same value as the
/// model's own `ModelNormalizationLimits::work_units` and
/// `PopulationAdmissionLimits::work_units` defaults.
pub const DEFAULT_WORK_UNITS: u64 = 16_777_216;

/// The ceilings [`TypeEnvironment::bounded`] admits object types under.
/// Every member is a real limit; zero is never "unlimited".
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeEnvironmentLimits {
    /// QSL FR-082's `ancestor_steps`: the most `supertypes` edges one
    /// conformance walk may follow, an edge count over the walk's closure
    /// and never a chain depth.
    pub ancestor_steps: u64,
    /// Cumulative work units admission may spend building the ancestor
    /// closure and flattening every object type's attributes. One unit is
    /// one ancestor or attribute copied into a type's set, or one field a
    /// lineage names. Running out is an
    /// [`EnvironmentLimitKind::WorkUnits`] limit.
    pub work_units: u64,
}

impl Default for TypeEnvironmentLimits {
    fn default() -> Self {
        Self {
            ancestor_steps: DEFAULT_ANCESTOR_STEPS,
            work_units: DEFAULT_WORK_UNITS,
        }
    }
}

/// The admission work meter: charges fail once `limit` units are spent.
struct WorkBudget<'a> {
    spent: u64,
    limit: u64,
    cancel: Option<&'a quire_exact::Cancel>,
}

impl<'a> WorkBudget<'a> {
    fn new(limit: u64, cancel: Option<&'a quire_exact::Cancel>) -> Self {
        Self {
            spent: 0,
            limit,
            cancel,
        }
    }

    /// Charge `units`, or the [`EnvironmentLimitKind::WorkUnits`] limit (QSL FR-082)
    /// once the budget would be passed, naming the cumulative total the
    /// refused charge would have reached.
    fn charge(&mut self, units: usize) -> Result<(), EnvironmentLimit> {
        let units = u64::try_from(units).unwrap_or(u64::MAX);
        // A cancelled handle (QSL FR-276) denies the charge as an exhausted
        // budget does; the caller that owns the handle reports the
        // cancellation.
        let cancelled = self.cancel.is_some_and(quire_exact::Cancel::poll);
        match self.spent.checked_add(units) {
            Some(spent) if spent <= self.limit && !cancelled => {
                self.spent = spent;
                Ok(())
            }
            _ => Err(EnvironmentLimit::new(
                EnvironmentLimitKind::WorkUnits,
                self.limit,
                u128::from(self.spent) + u128::from(units),
            )),
        }
    }
}

/// Why admitting one declaration stopped: a refusal of the input, or a
/// reached ceiling, which names no declaration.
enum Stopped {
    Refused(DeclarationCause),
    Limit(EnvironmentLimit),
    Storage(EnvironmentFailure),
}

impl From<DeclarationCause> for Stopped {
    fn from(cause: DeclarationCause) -> Self {
        Self::Refused(cause)
    }
}

impl From<EnvironmentLimit> for Stopped {
    fn from(limit: EnvironmentLimit) -> Self {
        Self::Limit(limit)
    }
}

impl From<EnvironmentFailure> for Stopped {
    fn from(failure: EnvironmentFailure) -> Self {
        Self::Storage(failure)
    }
}

impl Stopped {
    /// This stop as the admission's stage failure, a refusal naming
    /// `declaration`.
    fn at<R: ReservationPolicy>(
        self,
        declaration: &str,
        reserve: &R,
    ) -> Admission<EnvironmentFailure> {
        match self {
            Self::Refused(cause) => Ok(EnvironmentFailure::Refused(InvalidDeclaration {
                declaration: try_clone_str(declaration, reserve)?,
                cause,
            })),
            Self::Limit(limit) => Ok(EnvironmentFailure::Limit(limit)),
            Self::Storage(failure) => Ok(failure),
        }
    }
}

/// Which [`TypeEnvironmentLimits`] ceiling admission reached.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnvironmentLimitKind {
    /// `ancestor_steps`: one object type's conformance walk would follow
    /// more `supertypes` edges than the ceiling (QSL FR-082).
    AncestorSteps,
    /// `work_units`: the admission's cumulative ancestor-closure and
    /// flattening work.
    WorkUnits,
}

/// A [`TypeEnvironmentLimits`] ceiling reached during admission: which one,
/// its configured bound, and the counter the refused step would have
/// reached. It is wider than the bound because a cumulative total of two
/// `u64` counters can exceed `u64::MAX`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EnvironmentLimit {
    kind: EnvironmentLimitKind,
    configured_bound: u64,
    actual: u128,
}

impl EnvironmentLimit {
    /// The `kind` ceiling, configured at `configured_bound`, where the
    /// admission's counter reached `actual`.
    pub const fn new(kind: EnvironmentLimitKind, configured_bound: u64, actual: u128) -> Self {
        Self {
            kind,
            configured_bound,
            actual,
        }
    }

    /// The ceiling that was reached.
    pub const fn kind(&self) -> EnvironmentLimitKind {
        self.kind
    }

    /// The ceiling's configured bound.
    pub const fn configured_bound(&self) -> u64 {
        self.configured_bound
    }

    /// The value the refused step would have taken the counter to.
    pub const fn actual(&self) -> u128 {
        self.actual
    }
}

/// Native unit of a type-environment storage reservation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StorageUnit {
    /// Additional collection elements.
    AdditionalElements,
    /// Bytes.
    Bytes,
}

/// The exact amount submitted at a storage reservation boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StorageRequest {
    amount: usize,
    unit: StorageUnit,
}

impl StorageRequest {
    /// Make a request in its native unit.
    pub const fn new(amount: usize, unit: StorageUnit) -> Self {
        Self { amount, unit }
    }

    /// The amount submitted to the reservation.
    pub const fn amount(self) -> usize {
        self.amount
    }

    /// The request's native unit.
    pub const fn unit(self) -> StorageUnit {
        self.unit
    }
}

/// Why type-environment admission produced no environment (QSL FR-082,
/// FR-108). Storage failures carry no configured bound or declaration cause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnvironmentFailure {
    /// A ceiling was reached first.
    Limit(EnvironmentLimit),
    /// The declarations are refused.
    Refused(InvalidDeclaration),
    /// A representable reservation was denied.
    Allocation(StorageRequest),
    /// Reservation size arithmetic or representation overflowed.
    Capacity(StorageRequest),
}

impl EnvironmentFailure {
    /// The refusal, or the unchanged non-refusal failure.
    pub fn into_refused(self) -> Result<InvalidDeclaration, Self> {
        match self {
            Self::Refused(invalid) => Ok(invalid),
            other => Err(other),
        }
    }
}

/// Injects denial at the same reservation boundary used in production.
trait ReservationPolicy {
    fn permit(&self, request: StorageRequest) -> bool;

    fn phase(&self, _phase: StoragePhase) {}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StoragePhase {
    Registry,
    MemberTypes,
    Traversal,
    Ancestors,
    FieldIndex,
    Sealing,
    KeyJoin,
}

struct GlobalReservations;

impl ReservationPolicy for GlobalReservations {
    fn permit(&self, _request: StorageRequest) -> bool {
        true
    }
}

fn reserve_elements<T, R: ReservationPolicy>(
    values: &mut Vec<T>,
    additional: usize,
    policy: &R,
) -> Admission<()> {
    let request = StorageRequest::new(additional, StorageUnit::AdditionalElements);
    let required = values
        .len()
        .checked_add(additional)
        .ok_or(EnvironmentFailure::Capacity(request))?;
    let layout = Layout::array::<T>(required).map_err(|_| EnvironmentFailure::Capacity(request))?;
    if layout.size() > isize::MAX as usize {
        return Err(EnvironmentFailure::Capacity(request));
    }
    if required <= values.capacity() {
        return Ok(());
    }
    if !policy.permit(request) {
        return Err(EnvironmentFailure::Allocation(request));
    }
    values
        .try_reserve_exact(additional)
        .map_err(|_| EnvironmentFailure::Allocation(request))
}

fn try_push<T, R: ReservationPolicy>(values: &mut Vec<T>, value: T, policy: &R) -> Admission<()> {
    reserve_elements(values, 1, policy)?;
    values.push(value);
    Ok(())
}

fn try_extend_copy<T: Copy, R: ReservationPolicy>(
    values: &mut Vec<T>,
    source: &[T],
    policy: &R,
) -> Admission<()> {
    reserve_elements(values, source.len(), policy)?;
    values.extend_from_slice(source);
    Ok(())
}

fn reserve_bytes<R: ReservationPolicy>(
    value: &mut String,
    additional: usize,
    policy: &R,
) -> Admission<()> {
    let request = StorageRequest::new(additional, StorageUnit::Bytes);
    let required = value
        .len()
        .checked_add(additional)
        .ok_or(EnvironmentFailure::Capacity(request))?;
    if required > isize::MAX as usize {
        return Err(EnvironmentFailure::Capacity(request));
    }
    if required <= value.capacity() {
        return Ok(());
    }
    if !policy.permit(request) {
        return Err(EnvironmentFailure::Allocation(request));
    }
    value
        .try_reserve_exact(additional)
        .map_err(|_| EnvironmentFailure::Allocation(request))
}

fn try_clone_str<R: ReservationPolicy>(value: &str, policy: &R) -> Admission<String> {
    let mut owned = String::new();
    reserve_bytes(&mut owned, value.len(), policy)?;
    owned.push_str(value);
    Ok(owned)
}

fn try_clone_field_ref<R: ReservationPolicy>(value: &FieldRef, policy: &R) -> Admission<FieldRef> {
    Ok(FieldRef {
        owner: value.owner,
        name: try_clone_str(&value.name, policy)?,
    })
}

/// Ordered index whose growth is reserved before it changes. It has the
/// same ascending iteration and binary-search lookup as the old tree index.
#[derive(Clone, Debug, Eq, PartialEq)]
struct OrderedIndex<K, V> {
    entries: Vec<(K, V)>,
}

impl<K, V> Default for OrderedIndex<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<K: Ord, V> OrderedIndex<K, V> {
    fn get(&self, key: &K) -> Option<&V> {
        self.entries
            .binary_search_by(|(item, _)| item.cmp(key))
            .ok()
            .map(|at| &self.entries[at].1)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.entries
            .binary_search_by(|(item, _)| item.cmp(key))
            .ok()
            .map(|at| &mut self.entries[at].1)
    }

    fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.entries
            .binary_search_by(|(item, _)| item.cmp(key))
            .ok()
            .map(|at| self.entries.remove(at).1)
    }

    fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|(_, value)| value)
    }

    fn value_at(&self, position: usize) -> Option<&V> {
        self.entries.get(position).map(|(_, value)| value)
    }

    fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(key, _)| key)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn try_insert<R: ReservationPolicy>(
        &mut self,
        key: K,
        value: V,
        reserve: &R,
    ) -> Admission<Option<V>> {
        match self.entries.binary_search_by(|(item, _)| item.cmp(&key)) {
            Ok(at) => Ok(Some(core::mem::replace(&mut self.entries[at].1, value))),
            Err(at) => {
                reserve_elements(&mut self.entries, 1, reserve)?;
                self.entries.insert(at, (key, value));
                Ok(None)
            }
        }
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a OrderedIndex<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = core::iter::Map<core::slice::Iter<'a, (K, V)>, fn(&(K, V)) -> (&K, &V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().map(|(key, value)| (key, value))
    }
}

/// Type-environment admission's outcome (QSL FR-082).
pub type Admission<T> = Result<T, EnvironmentFailure>;

/// Why a declaration set is not admitted.
#[derive(Clone, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[error("declaration {declaration} refused: {cause:?}")]
pub struct InvalidDeclaration {
    /// The name of the declaration where the refusal originates.
    pub declaration: String,
    /// The typed cause.
    pub cause: DeclarationCause,
}

impl InvalidDeclaration {
    /// The `refused { code }` spelling.
    pub fn code(&self) -> &'static str {
        match self.cause {
            DeclarationCause::DuplicateKey
            | DeclarationCause::DuplicateMember(_)
            | DeclarationCause::UnknownDeclaration(_)
            | DeclarationCause::UnknownObjectType(_)
            | DeclarationCause::RedefinitionTarget(_)
            | DeclarationCause::RedefinitionConflict(_) => "invalid_semantic_graph",
            DeclarationCause::Type(_)
            | DeclarationCause::Recursion { .. }
            | DeclarationCause::GeneralizationCycle { .. }
            | DeclarationCause::RedefinitionWidens(_) => IllTyped::CODE,
        }
    }
}

/// Which recursion-rule subgraph has a cycle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RecursionEdges {
    /// A cycle of edges that start at tuple positions.
    Unnamed,
    /// A cycle of edges that pass no `?`, `Option` or minimum-zero collection.
    NonEscaping,
}

/// The typed cause of an [`InvalidDeclaration`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DeclarationCause {
    /// Two declarations share one node key.
    DuplicateKey,
    /// Two fields of one record, or two attributes of one object type's
    /// effective attribute set, share this name. An inherited field and a
    /// field of the same name that does not redefine it are two attributes.
    DuplicateMember(String),
    /// A field's `redefines` names no field of a proper ancestor of its
    /// owning object type, or a record field declares a `redefines`.
    RedefinitionTarget(FieldRef),
    /// Two attributes of one object type's effective set both stand for
    /// this field, and neither owner is a proper descendant of the other:
    /// QSpec FR-151's conflicting redefinitions of one member reaching a type.
    RedefinitionConflict(FieldRef),
    /// A field that redefines this one widens it: an optional redefiner of
    /// a required field, or a value type admitting a value the redefined
    /// field's type does not.
    RedefinitionWidens(FieldRef),
    /// A type names a key that is no declaration of the package.
    UnknownDeclaration(NodeKey),
    /// A declared supertype names an effective identity that is no admitted
    /// object type of the package.
    UnknownObjectType(EffectiveId),
    /// A member type is ill-typed: a `Reference<T>` target that is not an
    /// admitted model object type (`type-mismatch`), or an IEEE-bearing set,
    /// bag or ordered-set element type (`operator-ineligible`).
    Type(IllTypedCause),
    /// The recursion rule refuses this cycle of declaration names, whose first
    /// and last entries are the same declaration.
    Recursion {
        /// The offending subgraph.
        edges: RecursionEdges,
        /// The declaration names along the cycle.
        cycle: Vec<String>,
    },
    /// An object type's own declared `supertypes` form a
    /// cycle: a separate graph from [`Self::Recursion`]'s field-containment
    /// one -- generalization, not containment -- so it earns its own
    /// variant rather than reusing [`RecursionEdges`].
    GeneralizationCycle {
        /// The object-type names along the cycle, first and last equal.
        cycle: Vec<String>,
    },
}

/// One checked package's closed, admitted record, tuple and object-type
/// declarations.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeEnvironment {
    composites: OrderedIndex<NodeKey, CompositeDeclaration>,
    object_types: OrderedIndex<EffectiveId, ObjectTypeDeclaration>,
    /// Every object type's own proper ancestor set:
    /// transitive, not just direct, `supertypes`. Precomputed once in
    /// [`TypeEnvironment::new`], after the supertypes graph is known
    /// acyclic, so [`Self::conforms`] is a plain set lookup.
    ancestry: Ancestry,
    /// Every object type's effective attribute set: its own fields
    /// plus every ancestor's, less each field another field of the set
    /// redefines. Computed once in [`TypeEnvironment::bounded`]; an object's
    /// storage slots are exactly this set, so an attribute lookup never
    /// walks ancestors.
    effective: OrderedIndex<EffectiveId, Vec<usize>>,
    attribute_slots: Vec<AttributeSlot>,
    field_keys: Vec<(EffectiveId, usize)>,
    /// The units a `ValueType::Quantity` of this package names by id.
    units: UnitTable,
}

/// One containment edge of the recursion rule.
#[derive(Clone, Copy)]
struct Edge {
    target: NodeKey,
    named: bool,
    escapes: bool,
}

impl TypeEnvironment {
    /// Admit `composites` and `object_types` as one closed environment,
    /// under the QSL NFR-012 default ceilings ([`TypeEnvironmentLimits::default`]).
    /// See [`Self::bounded`].
    pub fn new(
        composites: impl IntoIterator<Item = CompositeDeclaration>,
        object_types: impl IntoIterator<Item = ObjectTypeDeclaration>,
    ) -> Admission<Self> {
        Self::bounded(composites, object_types, TypeEnvironmentLimits::default())
    }

    /// Admit `composites` and `object_types` as one closed environment.
    ///
    /// `limits.ancestor_steps` is the QSL FR-082 ceiling the model walks this
    /// package's conformance under at evaluation (the population binding's
    /// own `ancestor_steps`). An object type whose walk would follow more
    /// `supertypes` edges than that stops admission with an
    /// [`EnvironmentLimitKind::AncestorSteps`] limit (QSL FR-082). Check time is the stricter
    /// side: every conformance question the checker answers from this
    /// environment is one evaluation completes with the same verdict.
    ///
    /// `limits.work_units` bounds the whole admission's ancestor-closure
    /// and flattening work; running out is an
    /// [`EnvironmentLimitKind::WorkUnits`] limit.
    pub fn bounded(
        composites: impl IntoIterator<Item = CompositeDeclaration>,
        object_types: impl IntoIterator<Item = ObjectTypeDeclaration>,
        limits: TypeEnvironmentLimits,
    ) -> Admission<Self> {
        Self::bounded_with_reservations(composites, object_types, limits, None, &GlobalReservations)
    }

    /// [`Self::bounded`] with the caller's [`quire_exact::Cancel`] handle
    /// polled at every work charge (QSL FR-276). A cancelled handle stops
    /// admission with a [`EnvironmentLimitKind::WorkUnits`] limit, which the
    /// caller that owns the handle reads as its cancellation.
    pub fn bounded_with_cancel(
        composites: impl IntoIterator<Item = CompositeDeclaration>,
        object_types: impl IntoIterator<Item = ObjectTypeDeclaration>,
        limits: TypeEnvironmentLimits,
        cancel: &quire_exact::Cancel,
    ) -> Admission<Self> {
        Self::bounded_with_reservations(
            composites,
            object_types,
            limits,
            Some(cancel),
            &GlobalReservations,
        )
    }

    fn bounded_with_reservations<R: ReservationPolicy>(
        composites: impl IntoIterator<Item = CompositeDeclaration>,
        object_types: impl IntoIterator<Item = ObjectTypeDeclaration>,
        limits: TypeEnvironmentLimits,
        cancel: Option<&quire_exact::Cancel>,
        reserve: &R,
    ) -> Admission<Self> {
        let mut environment = Self::default();
        reserve.phase(StoragePhase::Registry);
        for declaration in composites {
            let refuse = |cause| -> Admission<EnvironmentFailure> {
                Ok(EnvironmentFailure::Refused(InvalidDeclaration {
                    declaration: try_clone_str(&declaration.name, reserve)?,
                    cause,
                }))
            };
            if let CompositeShape::Record(fields) = &declaration.shape {
                if let Some(name) = duplicate_name(fields, reserve)? {
                    return Err(refuse(DeclarationCause::DuplicateMember(try_clone_str(
                        name, reserve,
                    )?))?);
                }
                if let Some(target) = fields.iter().find_map(FieldDeclaration::redefines) {
                    return Err(refuse(DeclarationCause::RedefinitionTarget(
                        try_clone_field_ref(target, reserve)?,
                    ))?);
                }
            }
            if environment.composites.contains_key(&declaration.key) {
                return Err(refuse(DeclarationCause::DuplicateKey)?);
            }
            environment
                .composites
                .try_insert(declaration.key, declaration, reserve)?;
        }
        for declaration in object_types {
            let refuse = |cause| -> Admission<EnvironmentFailure> {
                Ok(EnvironmentFailure::Refused(InvalidDeclaration {
                    declaration: try_clone_str(&declaration.name, reserve)?,
                    cause,
                }))
            };
            if let Some(name) = duplicate_name(&declaration.attributes, reserve)? {
                return Err(refuse(DeclarationCause::DuplicateMember(try_clone_str(
                    name, reserve,
                )?))?);
            }
            if environment.object_types.contains_key(&declaration.key) {
                return Err(refuse(DeclarationCause::DuplicateKey)?);
            }
            environment
                .object_types
                .try_insert(declaration.key, declaration, reserve)?;
        }
        reserve.phase(StoragePhase::MemberTypes);
        environment.check_member_types(reserve)?;
        reserve.phase(StoragePhase::Traversal);
        environment.check_recursion(RecursionEdges::Unnamed, reserve)?;
        environment.check_recursion(RecursionEdges::NonEscaping, reserve)?;
        let mut budget = WorkBudget::new(limits.work_units, cancel);
        environment.check_supertypes(&mut budget, reserve)?;
        reserve.phase(StoragePhase::Ancestors);
        environment.ancestry = environment.compute_ancestors(&mut budget, reserve)?;
        environment
            .check_ancestor_steps(limits.ancestor_steps)
            .map_err(EnvironmentFailure::Limit)?;
        reserve.phase(StoragePhase::FieldIndex);
        let table = FieldTable::new(&environment.object_types, reserve)?;
        environment.check_redefinitions(&table, reserve)?;
        reserve.phase(StoragePhase::Sealing);
        let storage = environment.compute_effective(&table, &mut budget, reserve)?;
        environment.effective = storage.index;
        environment.attribute_slots = storage.slots;
        reserve.phase(StoragePhase::KeyJoin);
        reserve_elements(&mut environment.field_keys, table.fields.len(), reserve)?;
        for (owner, _, _, index) in &table.fields {
            environment.field_keys.push((*owner, *index));
        }
        Ok(environment)
    }

    /// The object type's effective attribute set, in slot order: its own
    /// fields in declaration order, then each direct supertype's slots in
    /// its own order, the supertypes in declaration order. Each attribute
    /// appears once; a field a more derived attribute of the set stands for
    /// is left out, and that attribute keeps its own place. `None` for a key
    /// that is no admitted object type.
    pub fn attributes(&self, object_type: EffectiveId) -> Option<AttributeSet<'_>> {
        self.effective.get(&object_type).map(|slots| AttributeSet {
            environment: self,
            slots,
        })
    }

    /// The attribute `name` resolves to in the object type's effective set.
    pub fn attribute(
        &self,
        object_type: EffectiveId,
        name: &str,
    ) -> Option<EffectiveAttribute<'_>> {
        self.attributes(object_type)?
            .iter()
            .find(|attribute| attribute.field().name() == name)
    }

    fn attribute_view(&self, index: usize) -> Option<EffectiveAttribute<'_>> {
        let slot = self.attribute_slots.get(index)?;
        let field = self
            .object_types
            .get(&slot.owner)?
            .attributes
            .get(slot.field_index)?;
        Some(EffectiveAttribute {
            environment: self,
            slot,
            field,
        })
    }

    fn field_position(&self, target: &FieldRef) -> Option<usize> {
        self.field_keys.iter().position(|(owner, field_index)| {
            *owner == target.owner
                && self
                    .object_types
                    .get(owner)
                    .and_then(|declaration| declaration.attributes.get(*field_index))
                    .is_some_and(|field| field.name() == target.name)
        })
    }

    /// This environment with `units` as its quantity unit table.
    pub fn with_units(mut self, units: UnitTable) -> Self {
        self.units = units;
        self
    }

    /// The quantity units this package's types name by id.
    pub fn units(&self) -> &UnitTable {
        &self.units
    }

    /// The admitted record or tuple declaration with this key.
    pub fn composite(&self, key: NodeKey) -> Option<&CompositeDeclaration> {
        self.composites.get(&key)
    }

    /// Every admitted record and tuple declaration in key order.
    pub fn composites(&self) -> impl Iterator<Item = &CompositeDeclaration> {
        self.composites.values()
    }

    /// The admitted object type with this effective identity.
    pub fn object_type(&self, key: EffectiveId) -> Option<&ObjectTypeDeclaration> {
        self.object_types.get(&key)
    }

    /// Every admitted object type in key order, for resolving a type name
    /// to `ValueType::Reference` alongside [`Self::composites`].
    pub fn object_types(&self) -> impl Iterator<Item = &ObjectTypeDeclaration> {
        self.object_types.values()
    }

    /// Whether `sub` conforms to `sup`: reflexive
    /// (`sub == sup` always conforms), or `sup` is a proper ancestor of
    /// `sub` in the admitted supertypes graph. `false` for either key
    /// outside this environment's own admitted object types, never a panic.
    pub fn conforms(&self, sub: EffectiveId, sup: EffectiveId) -> bool {
        sub == sup || self.ancestry.is_ancestor(sub, sup)
    }

    /// Whether `value_type` admits `value` under this environment's
    /// conformance: a `Reference<T>` admits a reference whose object's
    /// most-specific type conforms to `T` (QSpec FR-151, [`Self::conforms`]);
    /// every other pair is [`ValueType::admits`].
    pub fn admits(&self, value_type: &ValueType, value: &Value) -> bool {
        match (value_type, value) {
            (ValueType::Reference(declared), Value::Reference(reference)) => {
                self.conforms(reference.object_type(), *declared)
            }
            _ => value_type.admits(value),
        }
    }

    /// Check a type named outside a declaration (a parameter or result type):
    /// every named declaration exists, every `Reference<T>` names a model
    /// object type, and no set, bag or ordered set has an IEEE-bearing element
    /// type.
    pub fn check_type(&self, value_type: &ValueType) -> Result<(), IllTyped> {
        match self.type_refusal(value_type) {
            None => Ok(()),
            Some(DeclarationCause::Type(cause)) => Err(IllTyped { cause }),
            Some(_) => Err(IllTyped {
                cause: IllTypedCause::TypeMismatch,
            }),
        }
    }

    /// Whether `value_type` contains `Float32` or `Float64` at any depth,
    /// through record and tuple declarations included.
    pub fn contains_ieee(&self, value_type: &ValueType) -> bool {
        let mut visited = BTreeSet::new();
        let mut pending = vec![value_type];
        while let Some(value_type) = pending.pop() {
            match value_type {
                ValueType::Float(_) => return true,
                ValueType::Option(payload) => pending.push(payload),
                ValueType::Collection(collection) => pending.push(collection.element()),
                ValueType::Composite(key) => {
                    if !visited.insert(*key) {
                        continue;
                    }
                    match self
                        .composites
                        .get(key)
                        .map(|declaration| &declaration.shape)
                    {
                        Some(CompositeShape::Record(fields)) => {
                            pending.extend(fields.iter().map(FieldDeclaration::value_type));
                        }
                        Some(CompositeShape::Tuple(positions)) => pending.extend(positions),
                        None => {}
                    }
                }
                ValueType::Boolean
                | ValueType::Integer
                | ValueType::Int(_)
                | ValueType::Rational(_)
                | ValueType::Decimal(_)
                | ValueType::Quantity(_)
                | ValueType::Text(_)
                | ValueType::Uuid
                | ValueType::Timestamp
                | ValueType::Enum(_)
                | ValueType::Reference(_)
                | ValueType::Population(_) => {}
            }
        }
        false
    }

    /// The first refusal of one type's named declarations and element types.
    fn type_refusal(&self, value_type: &ValueType) -> Option<DeclarationCause> {
        let mut pending = vec![value_type];
        while let Some(value_type) = pending.pop() {
            match value_type {
                ValueType::Composite(key) if !self.composites.contains_key(key) => {
                    return Some(DeclarationCause::UnknownDeclaration(*key));
                }
                // QSpec FR-143: `T` in `Reference<T>` must name a model object
                // type; any other target is `type-mismatch`. The key is an
                // effective identity, so an admitted record or
                // tuple (keyed by `NodeKey`) can never satisfy it either.
                ValueType::Reference(key) if !self.object_types.contains_key(key) => {
                    return Some(DeclarationCause::Type(IllTypedCause::TypeMismatch));
                }
                // QSpec FR-153 names a population binding only as the direct
                // operand of `allInstances`/`lookup` (checked by
                // `Typer::all_instances`/`Typer::lookup` themselves, which
                // never route the population expression's own type through
                // this walk) and as a bare parameter type (bypassed by
                // `bind_parameters`, the one caller allowed to name it).
                // Every other named-type context -- an equality operand or
                // `Convert` target (`TypeEnvironment::check_equality`'s own
                // explicit refusal covers the former), an `Option` payload, a
                // collection element, or a record/tuple/object-type member --
                // refuses it here, never admitting a binding into a context
                // QSpec FR-153 never gives it Outputs for.
                ValueType::Population(_) => {
                    return Some(DeclarationCause::Type(IllTypedCause::OperatorIneligible));
                }
                ValueType::Option(payload) => pending.push(payload),
                ValueType::Collection(collection) => {
                    if collection.kind() != CollectionKind::Sequence
                        && self.contains_ieee(collection.element())
                    {
                        return Some(DeclarationCause::Type(IllTypedCause::OperatorIneligible));
                    }
                    pending.push(collection.element());
                }
                ValueType::Boolean
                | ValueType::Integer
                | ValueType::Int(_)
                | ValueType::Rational(_)
                | ValueType::Decimal(_)
                | ValueType::Float(_)
                | ValueType::Quantity(_)
                | ValueType::Text(_)
                | ValueType::Uuid
                | ValueType::Timestamp
                | ValueType::Enum(_)
                | ValueType::Composite(_)
                | ValueType::Reference(_) => {}
            }
        }
        None
    }

    fn contains_ieee_admission<R: ReservationPolicy>(
        &self,
        value_type: &ValueType,
        reserve: &R,
    ) -> Admission<bool> {
        let mut visited: OrderedIndex<NodeKey, ()> = OrderedIndex::default();
        let mut pending = Vec::new();
        try_push(&mut pending, value_type, reserve)?;
        while let Some(value_type) = pending.pop() {
            match value_type {
                ValueType::Float(_) => return Ok(true),
                ValueType::Option(payload) => try_push(&mut pending, payload, reserve)?,
                ValueType::Collection(collection) => {
                    try_push(&mut pending, collection.element(), reserve)?;
                }
                ValueType::Composite(key) => {
                    if visited.contains_key(key) {
                        continue;
                    }
                    visited.try_insert(*key, (), reserve)?;
                    match self
                        .composites
                        .get(key)
                        .map(|declaration| &declaration.shape)
                    {
                        Some(CompositeShape::Record(fields)) => {
                            for field in fields {
                                try_push(&mut pending, field.value_type(), reserve)?;
                            }
                        }
                        Some(CompositeShape::Tuple(positions)) => {
                            for position in positions {
                                try_push(&mut pending, position, reserve)?;
                            }
                        }
                        None => {}
                    }
                }
                ValueType::Boolean
                | ValueType::Integer
                | ValueType::Int(_)
                | ValueType::Rational(_)
                | ValueType::Decimal(_)
                | ValueType::Quantity(_)
                | ValueType::Text(_)
                | ValueType::Enum(_)
                | ValueType::Reference(_)
                | ValueType::Population(_) => {}
            }
        }
        Ok(false)
    }

    fn type_refusal_admission<R: ReservationPolicy>(
        &self,
        value_type: &ValueType,
        reserve: &R,
    ) -> Admission<Option<DeclarationCause>> {
        let mut pending = Vec::new();
        try_push(&mut pending, value_type, reserve)?;
        while let Some(value_type) = pending.pop() {
            match value_type {
                ValueType::Composite(key) if !self.composites.contains_key(key) => {
                    return Ok(Some(DeclarationCause::UnknownDeclaration(*key)));
                }
                ValueType::Reference(key) if !self.object_types.contains_key(key) => {
                    return Ok(Some(DeclarationCause::Type(IllTypedCause::TypeMismatch)));
                }
                ValueType::Population(_) => {
                    return Ok(Some(DeclarationCause::Type(
                        IllTypedCause::OperatorIneligible,
                    )));
                }
                ValueType::Option(payload) => try_push(&mut pending, payload, reserve)?,
                ValueType::Collection(collection) => {
                    if collection.kind() != CollectionKind::Sequence
                        && self.contains_ieee_admission(collection.element(), reserve)?
                    {
                        return Ok(Some(DeclarationCause::Type(
                            IllTypedCause::OperatorIneligible,
                        )));
                    }
                    try_push(&mut pending, collection.element(), reserve)?;
                }
                ValueType::Boolean
                | ValueType::Integer
                | ValueType::Int(_)
                | ValueType::Rational(_)
                | ValueType::Decimal(_)
                | ValueType::Float(_)
                | ValueType::Quantity(_)
                | ValueType::Text(_)
                | ValueType::Enum(_)
                | ValueType::Composite(_)
                | ValueType::Reference(_) => {}
            }
        }
        Ok(None)
    }

    fn check_member_types<R: ReservationPolicy>(&self, reserve: &R) -> Admission<()> {
        for declaration in self.composites.values() {
            let types: &[_] = match &declaration.shape {
                CompositeShape::Record(_) => &[],
                CompositeShape::Tuple(positions) => positions,
            };
            for value_type in types {
                if let Some(cause) = self.type_refusal_admission(value_type, reserve)? {
                    return Err(EnvironmentFailure::Refused(InvalidDeclaration {
                        declaration: try_clone_str(&declaration.name, reserve)?,
                        cause,
                    }));
                }
            }
            if let CompositeShape::Record(fields) = &declaration.shape {
                for field in fields {
                    if let Some(cause) = self.type_refusal_admission(field.value_type(), reserve)? {
                        return Err(EnvironmentFailure::Refused(InvalidDeclaration {
                            declaration: try_clone_str(&declaration.name, reserve)?,
                            cause,
                        }));
                    }
                }
            }
        }
        for declaration in self.object_types.values() {
            for field in &declaration.attributes {
                if let Some(cause) = self.type_refusal_admission(field.value_type(), reserve)? {
                    return Err(EnvironmentFailure::Refused(InvalidDeclaration {
                        declaration: try_clone_str(&declaration.name, reserve)?,
                        cause,
                    }));
                }
            }
        }
        Ok(())
    }

    /// The recursion-rule edges leaving one declaration.
    fn edges<R: ReservationPolicy>(
        declaration: &CompositeDeclaration,
        reserve: &R,
    ) -> Admission<Vec<Edge>> {
        let mut members: Vec<(&ValueType, bool, bool)> = Vec::new();
        match &declaration.shape {
            CompositeShape::Record(fields) => {
                for field in fields {
                    try_push(
                        &mut members,
                        (
                            field.value_type(),
                            true,
                            field.presence() == Presence::Optional,
                        ),
                        reserve,
                    )?;
                }
            }
            CompositeShape::Tuple(positions) => {
                for ty in positions {
                    try_push(&mut members, (ty, false, false), reserve)?;
                }
            }
        }
        let mut edges = Vec::new();
        for (value_type, named, escapes) in members {
            let mut pending = Vec::new();
            try_push(&mut pending, (value_type, escapes), reserve)?;
            while let Some((value_type, escapes)) = pending.pop() {
                match value_type {
                    ValueType::Composite(target) => try_push(
                        &mut edges,
                        Edge {
                            target: *target,
                            named,
                            escapes,
                        },
                        reserve,
                    )?,
                    ValueType::Option(payload) => try_push(&mut pending, (payload, true), reserve)?,
                    ValueType::Collection(collection) => try_push(
                        &mut pending,
                        (
                            collection.element(),
                            escapes || collection.bound().is_none_or(|bound| bound.minimum() == 0),
                        ),
                        reserve,
                    )?,
                    ValueType::Boolean
                    | ValueType::Integer
                    | ValueType::Int(_)
                    | ValueType::Rational(_)
                    | ValueType::Decimal(_)
                    | ValueType::Float(_)
                    | ValueType::Quantity(_)
                    | ValueType::Text(_)
                    | ValueType::Uuid
                    | ValueType::Timestamp
                    | ValueType::Enum(_)
                    | ValueType::Reference(_)
                    | ValueType::Population(_) => {}
                }
            }
        }
        Ok(edges)
    }

    /// Refuse the first cycle, in declaration-key order, of one recursion-rule
    /// subgraph.
    fn check_recursion<R: ReservationPolicy>(
        &self,
        subgraph: RecursionEdges,
        reserve: &R,
    ) -> Admission<()> {
        let mut graph: OrderedIndex<NodeKey, Vec<NodeKey>> = OrderedIndex::default();
        for declaration in self.composites.values() {
            let mut targets = Vec::new();
            for edge in Self::edges(declaration, reserve)? {
                let included = match subgraph {
                    RecursionEdges::Unnamed => !edge.named,
                    RecursionEdges::NonEscaping => !edge.escapes,
                };
                if included {
                    try_push(&mut targets, edge.target, reserve)?;
                }
            }
            graph.try_insert(declaration.key, targets, reserve)?;
        }
        let name = |key: &NodeKey| {
            self.composites
                .get(key)
                .map_or("", |declaration| declaration.name.as_str())
        };
        let mut finished: OrderedIndex<NodeKey, ()> = OrderedIndex::default();
        for root in graph.keys() {
            if finished.contains_key(root) {
                continue;
            }
            let mut path: Vec<(NodeKey, usize)> = Vec::new();
            try_push(&mut path, (*root, 0), reserve)?;
            while let Some((node, next)) = path.last_mut() {
                let node = *node;
                let Some(target) = graph.get(&node).and_then(|targets| targets.get(*next)) else {
                    finished.try_insert(node, (), reserve)?;
                    path.pop();
                    continue;
                };
                *next += 1;
                if let Some(start) = path.iter().position(|(on_path, _)| on_path == target) {
                    let mut cycle = Vec::new();
                    for (key, _) in path.iter().skip(start) {
                        try_push(&mut cycle, try_clone_str(name(key), reserve)?, reserve)?;
                    }
                    try_push(&mut cycle, try_clone_str(name(target), reserve)?, reserve)?;
                    return Err(EnvironmentFailure::Refused(InvalidDeclaration {
                        declaration: try_clone_str(name(target), reserve)?,
                        cause: DeclarationCause::Recursion {
                            edges: subgraph,
                            cycle,
                        },
                    }));
                }
                if !finished.contains_key(target) {
                    try_push(&mut path, (*target, 0), reserve)?;
                }
            }
        }
        Ok(())
    }

    /// Every object type's own declared `supertypes`, keyed by its own key
    /// every entry must itself name an admitted object
    /// type, and the whole graph must be acyclic -- refuses the first cycle
    /// found, in declaration-key order, exactly as [`Self::check_recursion`]
    /// does for the separate field-containment graph.
    ///
    /// Each type is marked while it is on the walk's path, so a back edge is
    /// found without scanning the path, and `budget` is charged each edge
    /// the walk follows.
    fn check_supertypes<R: ReservationPolicy>(
        &self,
        budget: &mut WorkBudget,
        reserve: &R,
    ) -> Admission<()> {
        let name = |key: &EffectiveId| {
            self.object_types
                .get(key)
                .map_or("", |declaration| declaration.name.as_str())
        };
        for declaration in self.object_types.values() {
            for supertype in &declaration.supertypes {
                if !self.object_types.contains_key(supertype) {
                    return Err(EnvironmentFailure::Refused(InvalidDeclaration {
                        declaration: try_clone_str(&declaration.name, reserve)?,
                        cause: DeclarationCause::UnknownObjectType(*supertype),
                    }));
                }
            }
        }
        let mut finished: OrderedIndex<EffectiveId, ()> = OrderedIndex::default();
        let mut on_path: OrderedIndex<EffectiveId, ()> = OrderedIndex::default();
        for root in self.object_types.keys() {
            if finished.contains_key(root) {
                continue;
            }
            let mut path: Vec<(EffectiveId, usize)> = Vec::new();
            try_push(&mut path, (*root, 0), reserve)?;
            on_path.try_insert(*root, (), reserve)?;
            while let Some((node, next)) = path.last_mut() {
                let node = *node;
                let Some(target) = self
                    .object_types
                    .get(&node)
                    .and_then(|declaration| declaration.supertypes.get(*next))
                else {
                    finished.try_insert(node, (), reserve)?;
                    on_path.remove(&node);
                    path.pop();
                    continue;
                };
                *next += 1;
                budget.charge(1).map_err(EnvironmentFailure::Limit)?;
                if on_path.contains_key(target) {
                    // Found once, on the refusal path only.
                    let start = path
                        .iter()
                        .position(|(on_path, _)| on_path == target)
                        .unwrap_or_default();
                    let mut cycle = Vec::new();
                    for (key, _) in path.iter().skip(start) {
                        try_push(&mut cycle, try_clone_str(name(key), reserve)?, reserve)?;
                    }
                    try_push(&mut cycle, try_clone_str(name(target), reserve)?, reserve)?;
                    return Err(EnvironmentFailure::Refused(InvalidDeclaration {
                        declaration: try_clone_str(name(target), reserve)?,
                        cause: DeclarationCause::GeneralizationCycle { cycle },
                    }));
                }
                if !finished.contains_key(target) {
                    on_path.try_insert(*target, (), reserve)?;
                    try_push(&mut path, (*target, 0), reserve)?;
                }
            }
        }
        Ok(())
    }

    /// Every object type's own proper ancestor set, transitively closed over
    /// the admitted (already known acyclic, see [`Self::check_supertypes`])
    /// supertypes graph. An explicit stack, never native recursion, mirroring
    /// [`Self::check_recursion`]'s own bounded walk: a node's ancestors are
    /// folded in only after every direct supertype's own ancestors are
    /// already known (a post-order finish), so each node is visited once and
    /// the walk is bounded by the object-type count, not call-stack depth.
    ///
    /// Each type's set is a copy of its supertypes' sets, so the whole
    /// closure costs the sum of every type's ancestor count; `budget` is
    /// charged that, a supertype and its ancestors at a time, before the
    /// copy is made.
    fn compute_ancestors<R: ReservationPolicy>(
        &self,
        budget: &mut WorkBudget,
        reserve: &R,
    ) -> Admission<Ancestry> {
        // Positions are `u32`: a package with more object types than that
        // could never be flattened within any budget either. Flattening
        // charges at least one unit a type, so the count is the least the
        // budget would have to reach.
        if u32::try_from(self.object_types.len()).is_err() {
            let types = u128::try_from(self.object_types.len()).unwrap_or(u128::MAX);
            return Err(EnvironmentFailure::Limit(EnvironmentLimit::new(
                EnvironmentLimitKind::WorkUnits,
                budget.limit,
                types.max(u128::from(budget.limit) + 1),
            )));
        }
        let mut positions = OrderedIndex::default();
        for (key, position) in self.object_types.keys().zip(0_u32..) {
            positions.try_insert(*key, position, reserve)?;
        }
        let mut ancestors: Vec<Option<Vec<u32>>> = Vec::new();
        reserve_elements(&mut ancestors, self.object_types.len(), reserve)?;
        ancestors.resize_with(self.object_types.len(), || None);
        let finished = |ancestors: &[Option<Vec<u32>>], key: &EffectiveId| {
            positions
                .get(key)
                .and_then(|position| ancestors.get(*position as usize))
                .is_some_and(Option::is_some)
        };
        for root in self.object_types.values() {
            if finished(&ancestors, &root.key) {
                continue;
            }
            let mut path: Vec<(&ObjectTypeDeclaration, usize)> = Vec::new();
            try_push(&mut path, (root, 0), reserve)?;
            while let Some((node, next)) = path.last_mut() {
                let node = *node;
                if let Some(target) = node.supertypes.get(*next) {
                    *next += 1;
                    if !finished(&ancestors, target) {
                        if let Some(general) = self.object_types.get(target) {
                            try_push(&mut path, (general, 0), reserve)?;
                        }
                    }
                    continue;
                }
                path.pop();
                let own = Self::own_ancestors(node, &positions, &ancestors, budget, reserve)?;
                if let Some(slot) = positions
                    .get(&node.key)
                    .and_then(|position| ancestors.get_mut(*position as usize))
                {
                    *slot = Some(own);
                }
            }
        }
        let mut complete = Vec::new();
        reserve_elements(&mut complete, ancestors.len(), reserve)?;
        complete.extend(ancestors.into_iter().map(Option::unwrap_or_default));
        Ok(Ancestry {
            positions,
            ancestors: complete,
        })
    }

    /// `node`'s proper ancestors, ascending, from its direct supertypes'
    /// finished sets. `budget` is charged each supertype plus its ancestor
    /// count before that set is copied; with several supertypes, the
    /// gathered positions are sorted and deduplicated once, and that sort
    /// is charged too (`k` units a doubling of the `k` gathered).
    fn own_ancestors<R: ReservationPolicy>(
        node: &ObjectTypeDeclaration,
        positions: &OrderedIndex<EffectiveId, u32>,
        ancestors: &[Option<Vec<u32>>],
        budget: &mut WorkBudget,
        reserve: &R,
    ) -> Admission<Vec<u32>> {
        let mut own: Vec<u32> = Vec::new();
        for supertype in &node.supertypes {
            let Some(position) = positions.get(supertype).copied() else {
                continue;
            };
            let further = ancestors
                .get(position as usize)
                .and_then(Option::as_deref)
                .unwrap_or_default();
            budget
                .charge(further.len().saturating_add(1))
                .map_err(EnvironmentFailure::Limit)?;
            try_push(&mut own, position, reserve)?;
            try_extend_copy(&mut own, further, reserve)?;
        }
        match node.supertypes.len() {
            // No supertype, or one: its set is already ascending, and only
            // its own position needs placing.
            0 => {}
            1 => {
                own.rotate_left(1);
                if let Some(position) = own.pop() {
                    if let Err(at) = own.binary_search(&position) {
                        own.insert(at, position);
                    }
                }
            }
            _ => {
                let gathered = own.len();
                let doublings = usize::try_from(gathered.max(1).ilog2())
                    .unwrap_or(usize::MAX)
                    .saturating_add(1);
                budget
                    .charge(gathered.saturating_mul(doublings))
                    .map_err(EnvironmentFailure::Limit)?;
                own.sort_unstable();
                own.dedup();
            }
        }
        Ok(own)
    }

    /// Stop at the first object type, in key order, whose QSL FR-082 conformance
    /// walk follows more than `limit` `supertypes` edges: an
    /// [`EnvironmentLimitKind::AncestorSteps`] limit whose actual counter is
    /// the ceiling plus one, the edge the walk would stop at. The model's
    /// walk from `S` (`ModelIndex::conforms`) follows each declared edge of
    /// `S` and of each distinct ancestor once, and stops early only when it
    /// meets its target, so that sum is the most any walk from `S` follows.
    /// Admitting only types within `limit` makes every walk from an admitted
    /// type complete under the same ceiling at evaluation.
    fn check_ancestor_steps(&self, limit: u64) -> Result<(), EnvironmentLimit> {
        let edges = |position: u32| {
            self.object_types
                .value_at(position as usize)
                .map(|declaration| u64::try_from(declaration.supertypes.len()).unwrap_or(u64::MAX))
        };
        for declaration in self.object_types.values() {
            let own = self
                .ancestry
                .positions
                .get(&declaration.key)
                .and_then(|position| edges(*position))
                .unwrap_or(0);
            let followed = self
                .ancestry
                .of(declaration.key)
                .unwrap_or_default()
                .iter()
                .filter_map(|position| edges(*position))
                .fold(own, |total, edges| total.saturating_add(edges));
            if followed > limit {
                return Err(EnvironmentLimit::new(
                    EnvironmentLimitKind::AncestorSteps,
                    limit,
                    u128::from(limit) + 1,
                ));
            }
        }
        Ok(())
    }

    /// Refuse the first object-type field, in key then declaration order,
    /// whose `redefines` names no field of a proper ancestor of its owner
    /// (QSpec FR-151: the target must be inherited by the owning type).
    fn check_redefinitions<R: ReservationPolicy>(
        &self,
        table: &FieldTable<'_>,
        reserve: &R,
    ) -> Admission<()> {
        for declaration in self.object_types.values() {
            for target in declaration
                .attributes
                .iter()
                .filter_map(FieldDeclaration::redefines)
            {
                let inherited = self.ancestry.is_ancestor(declaration.key, target.owner)
                    && table.position(target).is_some();
                if !inherited {
                    return Err(EnvironmentFailure::Refused(InvalidDeclaration {
                        declaration: try_clone_str(&declaration.name, reserve)?,
                        cause: DeclarationCause::RedefinitionTarget(try_clone_field_ref(
                            target, reserve,
                        )?),
                    }));
                }
            }
        }
        Ok(())
    }

    /// The first field `attribute` stands in for that it widens: `deref(r).f`
    /// through that field's owner reads `attribute`'s slot, so its presence
    /// must be required where that field's is, and every value its type
    /// admits that field's type must admit ([`Self::narrows`]). Otherwise
    /// the read would yield a value its checked type does not describe.
    fn widened<R: ReservationPolicy>(
        &self,
        index: usize,
        slots: &[AttributeSlot],
        table: &FieldTable<'_>,
        reserve: &R,
    ) -> Admission<Option<FieldRef>> {
        let Some(attribute) = slots.get(index) else {
            return Ok(None);
        };
        let Some(field) = table.field(attribute.identity) else {
            return Ok(None);
        };
        let found = attribute.members.iter().find(|member| {
            table.field(**member).is_some_and(|redefined| {
                (field.presence == Presence::Optional && redefined.presence == Presence::Required)
                    || !Self::narrows(&field.value_type, &redefined.value_type)
            })
        });
        match found {
            Some(member) => table.reference(*member, reserve),
            None => Ok(None),
        }
    }

    /// Whether every value `narrower` admits, `wider` admits *and*
    /// evaluation's `ValueType::admits` accepts a value of `narrower` where
    /// `wider` is declared: the same type, or an `Int` interval within
    /// `Integer` or within a wider `Int` interval. A reference field is
    /// redefined only with the identical reference type, since `admits`
    /// matches a reference's object type exactly. Any other pair is refused,
    /// never guessed.
    fn narrows(narrower: &ValueType, wider: &ValueType) -> bool {
        match (narrower, wider) {
            (narrower, wider) if narrower == wider => true,
            (ValueType::Int(_), ValueType::Integer) => true,
            (ValueType::Int(inner), ValueType::Int(outer)) => {
                outer.contains(inner.lower()) && outer.contains(inner.upper())
            }
            _ => false,
        }
    }

    /// Every object type's effective attribute set, flattened once.
    ///
    /// This applies QSpec FR-151's phase-4 redefinition result
    /// (`quire.model.normalize.redefine/v1`) to the `redefines` links a
    /// producer copies from the normalized domain package: a redefined field
    /// stays declared but is hidden in every type that also has its
    /// redefiner; when several redefinitions of one field reach a type, only
    /// the one whose owner is a proper descendant of every other stays
    /// exposed, and it stands for the others' slots too. Two exposed
    /// attributes that still stand for one field conflict
    /// ([`DeclarationCause::RedefinitionConflict`]), and two exposed
    /// attributes of one name are two fields, not a redefinition
    /// ([`DeclarationCause::DuplicateMember`]).
    ///
    /// Types are flattened supertypes first (an explicit post-order stack),
    /// each from its own fields and its direct supertypes' finished sets, so
    /// no type re-walks its ancestors and an inherited attribute is shared,
    /// not copied. `budget` is charged every attribute and lineage field a
    /// type's flattening reads.
    fn compute_effective<R: ReservationPolicy>(
        &self,
        table: &FieldTable<'_>,
        budget: &mut WorkBudget,
        reserve: &R,
    ) -> Admission<EffectiveStorage> {
        let mut effective: OrderedIndex<EffectiveId, Vec<usize>> = OrderedIndex::default();
        let mut slots = Vec::new();
        let mut scratch = Scratch::new(table.len(), table.name_count(), reserve)?;
        for (root, declaration) in &self.object_types {
            if effective.contains_key(root) {
                continue;
            }
            let mut path: Vec<(&ObjectTypeDeclaration, usize)> = Vec::new();
            try_push(&mut path, (declaration, 0), reserve)?;
            while let Some((node, next)) = path.last_mut() {
                let node = *node;
                if let Some(supertype) = node.supertypes.get(*next) {
                    *next += 1;
                    if !effective.contains_key(supertype) {
                        if let Some(general) = self.object_types.get(supertype) {
                            try_push(&mut path, (general, 0), reserve)?;
                        }
                    }
                    continue;
                }
                path.pop();
                let attributes = match self.flatten(
                    node,
                    &effective,
                    table,
                    &mut scratch,
                    budget,
                    &mut slots,
                    reserve,
                ) {
                    Ok(attributes) => attributes,
                    Err(stopped) => return Err(stopped.at(&node.name, reserve)?),
                };
                effective.try_insert(node.key, attributes, reserve)?;
            }
        }
        Ok(EffectiveStorage {
            index: effective,
            slots,
        })
    }

    /// One object type's effective attribute set, from its own fields and
    /// its direct supertypes' already flattened sets.
    ///
    /// Every attribute standing for a common field is one redefinition
    /// group (a union-find over the fields their lineages name). A group of
    /// one is kept as it is. A larger group keeps only the attribute whose
    /// owner is a proper descendant of every other member's owner, with the
    /// members' lineages merged into it; a group with no such member is
    /// QSpec FR-151's conflict.
    fn flatten<R: ReservationPolicy>(
        &self,
        declaration: &ObjectTypeDeclaration,
        effective: &OrderedIndex<EffectiveId, Vec<usize>>,
        table: &FieldTable<'_>,
        scratch: &mut Scratch,
        budget: &mut WorkBudget,
        slots: &mut Vec<AttributeSlot>,
        reserve: &R,
    ) -> Result<Vec<usize>, Stopped> {
        scratch.next_type();
        let mut candidates: Vec<usize> = Vec::new();
        let mut fresh: Vec<bool> = Vec::new();
        for (field_index, field) in declaration.attributes.iter().enumerate() {
            let attribute = own_attribute(
                declaration.key,
                field_index,
                field,
                table,
                budget,
                slots,
                reserve,
            )?;
            if let Some(slot) = slots.get(attribute) {
                scratch.claim_identity(slot.identity, candidates.len());
            }
            try_push(&mut candidates, attribute, reserve)?;
            try_push(&mut fresh, true, reserve)?;
        }
        for supertype in &declaration.supertypes {
            let inherited = effective.get(supertype).map_or(&[][..], Vec::as_slice);
            budget.charge(inherited.len())?;
            for &attribute in inherited {
                // A diamond passes one field down two paths: keep it once,
                // with both paths' lineages when they differ.
                let Some(inherited_slot) = slots.get(attribute) else {
                    continue;
                };
                match scratch.identity(inherited_slot.identity) {
                    None => {
                        scratch.claim_identity(inherited_slot.identity, candidates.len());
                        try_push(&mut candidates, attribute, reserve)?;
                        try_push(&mut fresh, false, reserve)?;
                    }
                    Some(kept) => {
                        let Some(&existing) = candidates.get(kept) else {
                            continue;
                        };
                        budget.charge(inherited_slot.members.len())?;
                        if existing == attribute
                            || slots
                                .get(existing)
                                .is_some_and(|slot| slot.members == inherited_slot.members)
                        {
                            continue;
                        }
                        let merged =
                            merge(existing, [existing, attribute], slots, budget, reserve)?;
                        if let (Some(slot), Some(flag)) =
                            (candidates.get_mut(kept), fresh.get_mut(kept))
                        {
                            *slot = merged;
                            *flag = true;
                        }
                    }
                }
            }
        }

        let mut groups = UnionFind::new(candidates.len(), reserve)?;
        for (index, &attribute) in candidates.iter().enumerate() {
            let Some(slot) = slots.get(attribute) else {
                continue;
            };
            budget.charge(slot.members.len())?;
            for &member in &slot.members {
                match scratch.holder(member) {
                    Some(holder) => groups.union(holder, index, member),
                    None => scratch.hold(member, index),
                }
            }
        }

        let mut members_of: OrderedIndex<usize, Vec<usize>> = OrderedIndex::default();
        let mut roots = Vec::new();
        for index in 0..candidates.len() {
            let root = groups.find(index);
            try_push(&mut roots, root, reserve)?;
            if groups.size(root) > 1 {
                if let Some(members) = members_of.get_mut(&root) {
                    try_push(members, index, reserve)?;
                } else {
                    let mut members = Vec::new();
                    try_push(&mut members, index, reserve)?;
                    members_of.try_insert(root, members, reserve)?;
                }
            }
        }
        // Each group's surviving position, and its (possibly merged) attribute.
        let mut survivors: OrderedIndex<usize, (usize, usize)> = OrderedIndex::default();
        for (root, members) in &members_of {
            budget.charge(members.len())?;
            let owner = |index: usize| {
                candidates
                    .get(index)
                    .and_then(|slot| slots.get(*slot))
                    .map(|slot| slot.owner)
            };
            let properly = |sub: usize, sup: usize| match (owner(sub), owner(sup)) {
                (Some(sub), Some(sup)) => sub != sup && self.conforms(sub, sup),
                _ => false,
            };
            let mut best = members.first().copied().unwrap_or(*root);
            for &member in members {
                if properly(member, best) {
                    best = member;
                }
            }
            let conflict = members
                .iter()
                .any(|&member| member != best && !properly(best, member));
            let Some(&winner) = candidates.get(best) else {
                continue;
            };
            if conflict {
                // A group of two or more was joined over a common field, so
                // `shared` is always set; the winner's own field is named
                // otherwise, never an invented one.
                let mut shared = match groups.shared(*root) {
                    Some(position) => table.reference(position, reserve)?,
                    None => None,
                };
                if shared.is_none() {
                    if let Some(slot) = slots.get(winner) {
                        shared = table.reference(slot.identity, reserve)?;
                    }
                }
                let Some(shared) = shared else {
                    continue;
                };
                return Err(DeclarationCause::RedefinitionConflict(shared).into());
            }
            let merged = merge(
                winner,
                members
                    .iter()
                    .filter_map(|&member| candidates.get(member).copied()),
                slots,
                budget,
                reserve,
            )?;
            if let Some(flag) = fresh.get_mut(best) {
                *flag |= merged != winner;
            }
            survivors.try_insert(*root, (best, merged), reserve)?;
        }

        let mut attributes = Vec::new();
        let mut checked = Vec::new();
        for (index, attribute) in candidates.into_iter().enumerate() {
            let root = roots.get(index).copied().unwrap_or(index);
            let is_fresh = fresh.get(index).copied().unwrap_or(true);
            match survivors.get(&root) {
                Some((best, merged)) if *best == index => {
                    try_push(&mut attributes, *merged, reserve)?;
                    try_push(&mut checked, is_fresh, reserve)?;
                }
                Some(_) => {}
                None => {
                    try_push(&mut attributes, attribute, reserve)?;
                    try_push(&mut checked, is_fresh, reserve)?;
                }
            }
        }
        // An attribute a supertype already admitted unchanged was checked
        // there; only this type's own and merged attributes can widen.
        for (&attribute, &fresh) in attributes.iter().zip(&checked) {
            if fresh {
                if let Some(widened) = self.widened(attribute, slots, table, reserve)? {
                    return Err(DeclarationCause::RedefinitionWidens(widened).into());
                }
            }
        }
        budget.charge(attributes.len())?;
        for &attribute in &attributes {
            let taken = table
                .name_of(
                    slots
                        .get(attribute)
                        .map_or(usize::MAX, |slot| slot.identity),
                )
                .is_some_and(|name| !scratch.claim_name(name));
            if taken {
                let name = slots
                    .get(attribute)
                    .and_then(|slot| table.field(slot.identity))
                    .map_or("", FieldDeclaration::name);
                return Err(
                    DeclarationCause::DuplicateMember(try_clone_str(name, reserve)?).into(),
                );
            }
        }
        Ok(attributes)
    }

    /// Construct a record from its supplied fields. An omitted `?` field is
    /// `absent`.
    pub fn record(
        &self,
        declaration: NodeKey,
        fields: Vec<(&str, FieldValue)>,
    ) -> Result<Value, ConstructionRefusal> {
        let Some(CompositeShape::Record(declared)) = self.shape(declaration) else {
            return refuse(Component::Value, ConstructionCause::UnknownDeclaration);
        };
        let slots = fill_slots(self, declared, fields)?;
        Ok(composite(declaration, slots))
    }

    /// Construct a tuple of exactly its declared arity.
    pub fn tuple(
        &self,
        declaration: NodeKey,
        positions: Vec<Value>,
    ) -> Result<Value, ConstructionRefusal> {
        let declared = self.tuple_positions(declaration, positions.len())?;
        if let Some(position) = declared
            .iter()
            .zip(&positions)
            .position(|(value_type, value)| !self.admits(value_type, value))
        {
            return refuse(
                Component::Position(position),
                ConstructionCause::TypeMismatch,
            );
        }
        let slots = positions.into_iter().map(FieldValue::Present).collect();
        Ok(composite(declaration, slots))
    }

    /// Evaluate a record value expression. Every construction refusal is
    /// decided before any field expression runs. Field expressions then run in
    /// declaration order, whatever the source order; the first one that does
    /// not complete becomes the outcome and no later one runs. A completed
    /// record charges `composite.result-retain`.
    pub fn evaluate_record(
        &self,
        declaration: NodeKey,
        fields: Vec<(&str, FieldExpression<'_>)>,
        meter: &mut Meter,
    ) -> Result<Outcome<Value>, ConstructionRefusal> {
        let Some(CompositeShape::Record(declared)) = self.shape(declaration) else {
            return refuse(Component::Value, ConstructionCause::UnknownDeclaration);
        };
        let mut supplied = match_names(declared, fields)?;
        let mut plan = Vec::with_capacity(declared.len());
        for field in declared {
            let expression = supplied.remove(field.name());
            let component = || Component::Field(field.name().to_owned());
            match (&expression, field.presence()) {
                (None, Presence::Required) => {
                    return refuse(component(), ConstructionCause::MissingField)
                }
                (Some(FieldExpression::Null), Presence::Required) => {
                    return refuse(component(), ConstructionCause::NullForRequiredField)
                }
                (None | Some(FieldExpression::Null | FieldExpression::Evaluate(_)), _) => {}
            }
            plan.push((field, expression));
        }
        let mut slots = Vec::with_capacity(plan.len());
        for (field, expression) in plan {
            let slot = match expression {
                None => FieldValue::Absent,
                Some(FieldExpression::Null) => FieldValue::Null,
                Some(FieldExpression::Evaluate(expression)) => {
                    match admitted(field.value_type(), expression(meter)) {
                        Ok(value) => FieldValue::Present(value),
                        Err(stop) => return Ok(outcome_from_stop(Err(stop))),
                    }
                }
            };
            slots.push(slot);
        }
        Ok(retain_composite(
            composite(declaration, slots.into_boxed_slice()),
            meter,
        ))
    }

    /// Evaluate a tuple call `T(e, ...)`: the arity is checked first, then the
    /// arguments run in position order under the first-stopped rule, then a
    /// completed tuple charges `composite.result-retain`.
    pub fn evaluate_tuple(
        &self,
        declaration: NodeKey,
        positions: Vec<Deferred<'_>>,
        meter: &mut Meter,
    ) -> Result<Outcome<Value>, ConstructionRefusal> {
        let declared = self.tuple_positions(declaration, positions.len())?;
        let mut slots = Vec::with_capacity(declared.len());
        for (value_type, expression) in declared.iter().zip(positions) {
            match admitted(value_type, expression(meter)) {
                Ok(value) => slots.push(FieldValue::Present(value)),
                Err(stop) => return Ok(outcome_from_stop(Err(stop))),
            }
        }
        Ok(retain_composite(
            composite(declaration, slots.into_boxed_slice()),
            meter,
        ))
    }

    fn tuple_positions(
        &self,
        declaration: NodeKey,
        supplied: usize,
    ) -> Result<&[ValueType], ConstructionRefusal> {
        let Some(CompositeShape::Tuple(declared)) = self.shape(declaration) else {
            return refuse(Component::Value, ConstructionCause::UnknownDeclaration);
        };
        if declared.len() != supplied {
            return refuse(
                Component::Value,
                ConstructionCause::WrongArity {
                    declared: declared.len(),
                    supplied,
                },
            );
        }
        Ok(declared)
    }

    fn shape(&self, declaration: NodeKey) -> Option<&CompositeShape> {
        self.composites.get(&declaration).map(|d| &d.shape)
    }
}

/// Every admitted object type's proper ancestors, as positions in key
/// order: four bytes an ancestor, so the closure of a deep chain stays
/// small.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Ancestry {
    positions: OrderedIndex<EffectiveId, u32>,
    /// By position: that type's proper ancestors, ascending.
    ancestors: Vec<Vec<u32>>,
}

impl Ancestry {
    fn of(&self, key: EffectiveId) -> Option<&[u32]> {
        let position = self.positions.get(&key)?;
        self.ancestors.get(*position as usize).map(Vec::as_slice)
    }

    /// Whether `ancestor` is a proper ancestor of `key`.
    fn is_ancestor(&self, key: EffectiveId, ancestor: EffectiveId) -> bool {
        match (self.of(key), self.positions.get(&ancestor)) {
            (Some(ancestors), Some(position)) => ancestors.binary_search(position).is_ok(),
            _ => false,
        }
    }
}

/// `field`'s own attribute, declared by `owner`: it stands for itself and
/// every field it redefines, transitively. [`TypeEnvironment::check_redefinitions`]
/// has admitted every link, and each one moves to a proper ancestor of an
/// acyclic graph; `budget` is charged each link all the same.
fn own_attribute<R: ReservationPolicy>(
    owner: EffectiveId,
    field_index: usize,
    field: &FieldDeclaration,
    table: &FieldTable<'_>,
    budget: &mut WorkBudget,
    slots: &mut Vec<AttributeSlot>,
    reserve: &R,
) -> Result<usize, Stopped> {
    let identity = match table.position_parts(owner, field.name()) {
        Some(identity) => identity,
        None => {
            return Err(DeclarationCause::RedefinitionTarget(FieldRef {
                owner,
                name: try_clone_str(field.name(), reserve)?,
            })
            .into())
        }
    };
    let mut members = Vec::new();
    try_push(&mut members, identity, reserve)?;
    let mut next = field.redefines();
    while let Some(target) = next {
        budget.charge(1)?;
        let position = match table.position(target) {
            Some(position) => position,
            None => {
                return Err(DeclarationCause::RedefinitionTarget(try_clone_field_ref(
                    target, reserve,
                )?)
                .into())
            }
        };
        try_push(&mut members, position, reserve)?;
        next = table.field(position).and_then(FieldDeclaration::redefines);
    }
    members.sort_unstable();
    members.dedup();
    let index = slots.len();
    try_push(
        slots,
        AttributeSlot {
            owner,
            field_index,
            members,
            identity,
        },
        reserve,
    )?;
    Ok(index)
}

/// `winner` standing for every field any of `group` stands for. `winner`
/// itself, shared, when it already does.
fn merge<R: ReservationPolicy>(
    winner: usize,
    group: impl IntoIterator<Item = usize>,
    slots: &mut Vec<AttributeSlot>,
    budget: &mut WorkBudget,
    reserve: &R,
) -> Result<usize, Stopped> {
    let mut members: Vec<usize> = Vec::new();
    for index in group {
        if let Some(attribute) = slots.get(index) {
            budget.charge(attribute.members.len())?;
            try_extend_copy(&mut members, &attribute.members, reserve)?;
        }
    }
    let Some(winner_slot) = slots.get(winner) else {
        return Ok(winner);
    };
    budget.charge(winner_slot.members.len())?;
    try_extend_copy(&mut members, &winner_slot.members, reserve)?;
    members.sort_unstable();
    members.dedup();
    if members == winner_slot.members {
        return Ok(winner);
    }
    let index = slots.len();
    try_push(
        slots,
        AttributeSlot {
            owner: winner_slot.owner,
            field_index: winner_slot.field_index,
            members,
            identity: winner_slot.identity,
        },
        reserve,
    )?;
    Ok(index)
}

/// Every object-type field of one admission, numbered in [`FieldRef`]
/// order (owner, then name), so a sorted list of positions and the sorted
/// list of the fields they name line up. Names are numbered too, for the
/// duplicate-member check.
struct FieldTable<'e> {
    /// By position: the owner, the declaration and the name's number.
    fields: Vec<(EffectiveId, &'e FieldDeclaration, usize, usize)>,
    names: usize,
}

impl<'e> FieldTable<'e> {
    fn new<R: ReservationPolicy>(
        object_types: &'e OrderedIndex<EffectiveId, ObjectTypeDeclaration>,
        reserve: &R,
    ) -> Admission<Self> {
        let mut fields = Vec::new();
        for declaration in object_types.values() {
            for (field_index, field) in declaration.attributes.iter().enumerate() {
                try_push(
                    &mut fields,
                    (declaration.key, field, 0, field_index),
                    reserve,
                )?;
            }
        }
        fields.sort_unstable_by(
            |(left_owner, left_field, _, _), (right_owner, right_field, _, _)| {
                (left_owner, left_field.name()).cmp(&(right_owner, right_field.name()))
            },
        );
        let mut names: Vec<&str> = Vec::new();
        reserve_elements(&mut names, fields.len(), reserve)?;
        names.extend(fields.iter().map(|(_, field, _, _)| field.name()));
        names.sort_unstable();
        names.dedup();
        for (_, field, number, _) in &mut fields {
            if let Ok(position) = names.binary_search(&field.name()) {
                *number = position;
            }
        }
        Ok(Self {
            fields,
            names: names.len(),
        })
    }

    fn len(&self) -> usize {
        self.fields.len()
    }

    fn name_count(&self) -> usize {
        self.names
    }

    fn position(&self, field: &FieldRef) -> Option<usize> {
        self.position_parts(field.owner, field.name.as_str())
    }

    fn position_parts(&self, owner: EffectiveId, name: &str) -> Option<usize> {
        self.fields
            .binary_search_by(|(stored_owner, declared, _, _)| {
                (stored_owner, declared.name()).cmp(&(&owner, name))
            })
            .ok()
    }

    fn field(&self, position: usize) -> Option<&'e FieldDeclaration> {
        self.fields.get(position).map(|(_, field, _, _)| *field)
    }

    fn name_of(&self, position: usize) -> Option<usize> {
        self.fields.get(position).map(|(_, _, name, _)| *name)
    }

    fn reference<R: ReservationPolicy>(
        &self,
        position: usize,
        reserve: &R,
    ) -> Admission<Option<FieldRef>> {
        self.fields
            .get(position)
            .map(|(owner, field, _, _)| {
                Ok(FieldRef {
                    owner: *owner,
                    name: try_clone_str(field.name(), reserve)?,
                })
            })
            .transpose()
    }
}

/// Per-type marks for [`TypeEnvironment::flatten`], reused across types:
/// a mark counts only when it carries the current type's epoch, so moving
/// to the next type clears every mark at once.
struct Scratch {
    epoch: u64,
    /// By field position: the candidate holding that field as its identity.
    identities: Vec<(u64, usize)>,
    /// By field position: the first candidate whose lineage names it.
    holders: Vec<(u64, usize)>,
    /// By name number: whether a kept attribute already has that name.
    names: Vec<u64>,
}

impl Scratch {
    fn new<R: ReservationPolicy>(fields: usize, names: usize, reserve: &R) -> Admission<Self> {
        let mut identities = Vec::new();
        reserve_elements(&mut identities, fields, reserve)?;
        identities.resize(fields, (0, 0));
        let mut holders = Vec::new();
        reserve_elements(&mut holders, fields, reserve)?;
        holders.resize(fields, (0, 0));
        let mut marks = Vec::new();
        reserve_elements(&mut marks, names, reserve)?;
        marks.resize(names, 0);
        Ok(Self {
            epoch: 0,
            identities,
            holders,
            names: marks,
        })
    }

    fn next_type(&mut self) {
        self.epoch = self.epoch.saturating_add(1);
    }

    fn marked(marks: &[(u64, usize)], epoch: u64, position: usize) -> Option<usize> {
        marks
            .get(position)
            .filter(|(mark, _)| *mark == epoch)
            .map(|(_, candidate)| *candidate)
    }

    fn identity(&self, position: usize) -> Option<usize> {
        Self::marked(&self.identities, self.epoch, position)
    }

    fn claim_identity(&mut self, position: usize, candidate: usize) {
        if let Some(mark) = self.identities.get_mut(position) {
            *mark = (self.epoch, candidate);
        }
    }

    fn holder(&self, position: usize) -> Option<usize> {
        Self::marked(&self.holders, self.epoch, position)
    }

    fn hold(&mut self, position: usize, candidate: usize) {
        if let Some(mark) = self.holders.get_mut(position) {
            *mark = (self.epoch, candidate);
        }
    }

    /// Mark `name` taken; `false` when it already was.
    fn claim_name(&mut self, name: usize) -> bool {
        match self.names.get_mut(name) {
            Some(mark) if *mark == self.epoch => false,
            Some(mark) => {
                *mark = self.epoch;
                true
            }
            None => true,
        }
    }
}

/// Redefinition groups: a union-find over one type's candidates, joined
/// whenever two lineages name a common field. Each group remembers the
/// first such field, for the conflict refusal.
struct UnionFind {
    parent: Vec<usize>,
    size: Vec<usize>,
    shared: Vec<Option<usize>>,
}

impl UnionFind {
    fn new<R: ReservationPolicy>(len: usize, reserve: &R) -> Admission<Self> {
        let mut parent = Vec::new();
        reserve_elements(&mut parent, len, reserve)?;
        parent.extend(0..len);
        let mut size = Vec::new();
        reserve_elements(&mut size, len, reserve)?;
        size.resize(len, 1);
        let mut shared = Vec::new();
        reserve_elements(&mut shared, len, reserve)?;
        shared.resize(len, None);
        Ok(Self {
            parent,
            size,
            shared,
        })
    }

    fn find(&mut self, mut node: usize) -> usize {
        loop {
            let parent = self.parent.get(node).copied().unwrap_or(node);
            if parent == node {
                return node;
            }
            let grandparent = self.parent.get(parent).copied().unwrap_or(parent);
            if let Some(slot) = self.parent.get_mut(node) {
                *slot = grandparent;
            }
            node = grandparent;
        }
    }

    /// Join `left`'s and `right`'s groups over the common field `field`.
    fn union(&mut self, left: usize, right: usize, field: usize) {
        let (left, right) = (self.find(left), self.find(right));
        if left == right {
            return;
        }
        let (small, large) = if self.size(left) < self.size(right) {
            (left, right)
        } else {
            (right, left)
        };
        let merged = self.size(small).saturating_add(self.size(large));
        let shared = self.shared(large).or(self.shared(small)).or(Some(field));
        if let Some(slot) = self.parent.get_mut(small) {
            *slot = large;
        }
        if let Some(slot) = self.size.get_mut(large) {
            *slot = merged;
        }
        if let Some(slot) = self.shared.get_mut(large) {
            *slot = shared;
        }
    }

    fn size(&self, root: usize) -> usize {
        self.size.get(root).copied().unwrap_or(1)
    }

    fn shared(&self, root: usize) -> Option<usize> {
        self.shared.get(root).copied().flatten()
    }
}

fn duplicate_name<'a, R: ReservationPolicy>(
    fields: &'a [FieldDeclaration],
    reserve: &R,
) -> Admission<Option<&'a str>> {
    let mut seen: OrderedIndex<&str, ()> = OrderedIndex::default();
    for field in fields {
        if seen.try_insert(field.name(), (), reserve)?.is_some() {
            return Ok(Some(field.name()));
        }
    }
    Ok(None)
}

/// The completed value of a deferred expression, which a checked program
/// guarantees is a member of `value_type`. Its only callers are
/// `evaluate_record`/`evaluate_tuple` below.
fn admitted(value_type: &ValueType, outcome: Outcome<Value>) -> Result<Value, Stop> {
    let value = outcome_into_stop(outcome)?;
    if value_type.admits(&value) {
        Ok(value)
    } else {
        Err(invariant(CheckedInvariantCause::DeferredResultNotAdmitted))
    }
}

/// The grammar's equality operators.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EqualityOperator {
    /// `=`.
    Equal,
    /// `!=`: the same schedule, retaining the negated Boolean.
    NotEqual,
}

impl EqualityOperator {
    fn comparison(self) -> ComparisonOperator {
        match self {
            Self::Equal => ComparisonOperator::Equal,
            Self::NotEqual => ComparisonOperator::NotEqual,
        }
    }
}

/// The static type of one equality operand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EqualityOperand {
    source: ValueType,
    target: Option<ValueType>,
}

impl EqualityOperand {
    /// An operand `e` of static type `source`.
    pub fn typed(source: ValueType) -> Self {
        Self {
            source,
            target: None,
        }
    }

    /// An operand `convert<target>(e)` for `e` of static type `source`.
    pub fn converted(source: ValueType, target: ValueType) -> Self {
        Self {
            source,
            target: Some(target),
        }
    }

    /// The `convert<target>` target, when the operand converts.
    pub fn target(&self) -> Option<&ValueType> {
        self.target.as_ref()
    }

    /// The comparison type.
    fn comparison_type(&self) -> &ValueType {
        self.target.as_ref().unwrap_or(&self.source)
    }
}

/// The schedule QSpec FR-149 selects from the common type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EqualitySchedule {
    /// A top-level text pair: the QSpec FR-141 text schedule.
    Text,
    /// A top-level enumeration pair: `enum.*`.
    Enum,
    /// A top-level quantity pair: the QSpec FR-142 comparison schedule.
    Quantity,
    /// Every other common type: the occurrence-pair plan.
    Plan,
}

/// A type-checked equality expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedEquality {
    operator: EqualityOperator,
    left: EqualityOperand,
    right: EqualityOperand,
    schedule: EqualitySchedule,
    /// The unit of every top-level quantity type the operands name, resolved
    /// at checking, so evaluation reads no package table.
    units: UnitTable,
    /// The checked `VariantId -> EnumValue`
    /// index, captured at checking so `Self::evaluate` needs no extra
    /// argument. Empty and never consulted unless `schedule` is
    /// `EqualitySchedule::Enum`. Filtered to only the compared
    /// operands' own `EnumShape` -- never the whole package's enum-member
    /// index -- so a checked package with many sizeable enums does not
    /// retain O(equality nodes x total enum members) in its checked IR.
    enum_members: EnumMemberIndex,
}

impl TypeEnvironment {
    /// Type-check `left op right` against this environment's unit table.
    /// Every refusal is made before any charge. `enum_members` is the
    /// checked `VariantId -> EnumValue` index:
    /// `TypeEnvironment` itself holds no enum declarations (those are the
    /// checker's scope's), so a
    /// caller checking an `Enum`-scheduled equality supplies it here, once,
    /// rather than [`CheckedEquality::evaluate`] taking it as an extra
    /// argument every caller -- including every non-enum test -- would
    /// otherwise need to thread through. An empty index is correct for any
    /// caller that never checks an enum equality.
    pub fn check_equality(
        &self,
        operator: EqualityOperator,
        left: EqualityOperand,
        right: EqualityOperand,
        enum_members: &EnumMemberIndex,
    ) -> Result<CheckedEquality, IllTyped> {
        self.check_equality_in(
            &UnitScope::new(&self.units),
            operator,
            left,
            right,
            &|shape: &EnumShape| enum_members.filtered(shape.variants()),
        )
    }

    /// [`Self::check_equality`] against one checking stage's units, which
    /// add the compound units its expressions formed. `enum_members` gives
    /// the member index of one compared enum shape: exactly its own
    /// members. a checker's scope answers it from a table built
    /// once per shape, so an equality does not copy its enum.
    pub fn check_equality_in(
        &self,
        units: &UnitScope<'_>,
        operator: EqualityOperator,
        left: EqualityOperand,
        right: EqualityOperand,
        enum_members: &dyn Fn(&EnumShape) -> EnumMemberIndex,
    ) -> Result<CheckedEquality, IllTyped> {
        let ill_typed = |cause| Err(IllTyped { cause });
        for operand in [&left, &right] {
            if let Some(target) = &operand.target {
                if !admits_equality_conversion(&operand.source, target, units) {
                    return ill_typed(IllTypedCause::TypeMismatch);
                }
            }
        }
        // Every top-level quantity type resolves, or the operands name a unit
        // this package has not admitted (as an unknown enum declaration is a
        // type mismatch at checking).
        let mut resolved = UnitTable::default();
        for value_type in [&left.source, &right.source]
            .into_iter()
            .chain(left.target.iter())
            .chain(right.target.iter())
        {
            if let ValueType::Quantity(id) = value_type {
                let Some(unit) = units.get(*id) else {
                    return ill_typed(IllTypedCause::TypeMismatch);
                };
                resolved.insert_held(*id, unit.clone());
            }
        }
        let (left_type, right_type) = (left.comparison_type(), right.comparison_type());
        if self.contains_ieee(left_type) || self.contains_ieee(right_type) {
            return ill_typed(IllTypedCause::OperatorIneligible);
        }
        let schedule = match (left_type, right_type) {
            (ValueType::Text(l), ValueType::Text(r)) if l.profile() != r.profile() => {
                return ill_typed(IllTypedCause::DistinctTextProfiles)
            }
            (ValueType::Text(_), ValueType::Text(_)) => EqualitySchedule::Text,
            (ValueType::Enum(l), ValueType::Enum(r)) if l != r => {
                return ill_typed(IllTypedCause::DistinctEnumDeclarations)
            }
            (ValueType::Enum(_), ValueType::Enum(_)) => EqualitySchedule::Enum,
            (ValueType::Quantity(l), ValueType::Quantity(r))
                if resolved
                    .get(*l)
                    .zip(resolved.get(*r))
                    .is_some_and(|(l, r)| !l.has_dimension_of(r)) =>
            {
                return ill_typed(IllTypedCause::IncompatibleDimensions)
            }
            (ValueType::Quantity(l), ValueType::Quantity(r)) if l != r => {
                return ill_typed(IllTypedCause::DistinctUnits)
            }
            (ValueType::Quantity(_), ValueType::Quantity(_)) => EqualitySchedule::Quantity,
            // QSpec FR-153 names a population binding only as the direct operand of
            // `allInstances`/`lookup`, never as an equality operand: refuse it
            // here rather than falling into the `l == r` plan schedule below,
            // which would otherwise accept `p == p` and only fail at
            // evaluation (`plan_pairs`'s own checked-invariant catch-all).
            (ValueType::Population(_), _) | (_, ValueType::Population(_)) => {
                return ill_typed(IllTypedCause::OperatorIneligible)
            }
            // QSpec FR-153-AC-6 / QSpec TC-198 L08: a `Reference<A>` and a
            // `Reference<B>` denote the same real object when one type
            // conforms to the other, as `lookup<A>(p, rb) = rb` does. The
            // plan compares the two references' full identity whatever
            // their static types, so this admits more pairs and decides
            // none differently.
            (ValueType::Reference(l), ValueType::Reference(r))
                if self.conforms(*l, *r) || self.conforms(*r, *l) =>
            {
                EqualitySchedule::Plan
            }
            (l, r) if l == r => EqualitySchedule::Plan,
            _ => return ill_typed(IllTypedCause::TypeMismatch),
        };
        // Retain only the compared enum declaration's own
        // members (`left_type`'s `EnumShape`, which schedule selection
        // above already confirmed equals `right_type`'s), not a clone of
        // the whole package's `enum_members` index. Computed before `left`
        // moves into the struct literal below, since `left_type` borrows it.
        let enum_members = match (schedule, left_type) {
            (EqualitySchedule::Enum, ValueType::Enum(shape)) => enum_members(shape),
            _ => EnumMemberIndex::default(),
        };
        Ok(CheckedEquality {
            operator,
            left,
            right,
            schedule,
            units: resolved,
            enum_members,
        })
    }
}

impl CheckedEquality {
    /// The selected schedule.
    pub fn schedule(&self) -> EqualitySchedule {
        self.schedule
    }

    /// Evaluate over the two completed operand values, left conversion first.
    /// Neither source value is changed. For the `Enum` schedule, a bare
    /// kernel `Value::Enum` carries only its `VariantId`
    /// and rank, so this resolves each side back to its full declaration/
    /// ordered/case data through the `enum_members` index captured at
    /// checking before calling [`compare_enum`],
    /// which needs none of the other schedules' declaration or unit data.
    pub fn evaluate(&self, left: &Value, right: &Value, meter: &mut Meter) -> Outcome<bool> {
        outcome_from_stop(self.run(left, right, meter))
    }

    fn run(&self, left: &Value, right: &Value, meter: &mut Meter) -> Result<bool, Stop> {
        let units = UnitScope::new(&self.units);
        let left = operand_value(&self.left, left, &units, meter)?;
        let right = operand_value(&self.right, right, &units, meter)?;
        let operator = self.operator.comparison();
        let scheduled = match (self.schedule, &left, &right) {
            (EqualitySchedule::Text, Value::Text(l), Value::Text(r)) => {
                compare_text(operator, l, r, meter)
            }
            (EqualitySchedule::Enum, Value::Enum(l), Value::Enum(r)) => {
                let (Some(l), Some(r)) = (
                    self.enum_members.resolve(l.variant()),
                    self.enum_members.resolve(r.variant()),
                ) else {
                    return Err(invariant(
                        CheckedInvariantCause::EqualityEnumVariantUnresolved,
                    ));
                };
                compare_enum(operator, l, r, meter)
            }
            (EqualitySchedule::Quantity, Value::Quantity(l), Value::Quantity(r)) => {
                let (Some(l), Some(r)) = (units.resolve(l), units.resolve(r)) else {
                    return Err(invariant(CheckedInvariantCause::EqualityUnitUnresolved));
                };
                compare_quantity(operator, l, r, meter)
            }
            (EqualitySchedule::Plan, _, _) => {
                let equal = outcome_into_stop(quire_exact::planned_equality(&left, &right, meter))?;
                return Ok(equal == (self.operator == EqualityOperator::Equal));
            }
            (
                EqualitySchedule::Text | EqualitySchedule::Enum | EqualitySchedule::Quantity,
                _,
                _,
            ) => return Err(invariant(CheckedInvariantCause::EqualityScheduleMismatch)),
        };
        outcome_into_stop(scheduled.map_err(|error| {
            invariant(CheckedInvariantCause::ScheduledComparisonRefused { cause: error.cause })
        })?)
    }
}

fn invariant(cause: CheckedInvariantCause) -> Stop {
    Stop::Refused(Refusal::CheckedInvariant { cause })
}

/// Whether (`source`, `target`) is a row of the closed QSpec FR-149
/// equality-conversion table, decided from declared bounds alone and, for a
/// quantity pair, from the units `units` resolves; an unresolved unit admits
/// no conversion.
pub fn admits_equality_conversion(
    source: &ValueType,
    target: &ValueType,
    units: &UnitScope<'_>,
) -> bool {
    match (source, target) {
        (source, target) if source == target => true,
        (ValueType::Int(interval), target) => {
            integer_source_admits(interval.lower(), interval.upper(), target)
        }
        (ValueType::Rational(from), ValueType::Rational(to)) => {
            to.numerator().lower() <= from.numerator().lower()
                && from.numerator().upper() <= to.numerator().upper()
                && to.denominator().lower() <= from.denominator().lower()
                && from.denominator().upper() <= to.denominator().upper()
        }
        (
            ValueType::Rational(from),
            target @ (ValueType::Integer | ValueType::Int(_) | ValueType::Decimal(_)),
        ) if from.denominator().upper() == &Integer::one() => {
            integer_source_admits(from.numerator().lower(), from.numerator().upper(), target)
        }
        (ValueType::Decimal(from), ValueType::Rational(to)) => {
            let zero = Integer::zero();
            to.numerator().lower() <= from.lower().min(&zero)
                && from.upper().max(&zero) <= to.numerator().upper()
                && to.denominator().lower() <= &Integer::one()
                && compare_shifted(
                    &Integer::one(),
                    u64::from(from.max_scale()),
                    to.denominator().upper(),
                )
                .is_le()
        }
        (ValueType::Decimal(from), ValueType::Decimal(to)) => {
            let zero = Integer::zero();
            to.min_scale() <= from.min_scale()
                && from.max_scale() <= to.max_scale()
                && if to.min_scale() == from.min_scale() {
                    to.lower() <= from.lower() && from.upper() <= to.upper()
                } else {
                    to.lower() <= from.lower().min(&zero) && from.upper().max(&zero) <= to.upper()
                }
        }
        (ValueType::Decimal(from), target @ (ValueType::Integer | ValueType::Int(_)))
            if from.max_scale() == 0 =>
        {
            integer_source_admits(from.lower(), from.upper(), target)
        }
        (ValueType::Quantity(from), ValueType::Quantity(to)) => units
            .get(*from)
            .zip(units.get(*to))
            .is_some_and(|(from, to)| from.converts_to(to)),
        _ => false,
    }
}

/// The rows for a source `Int[lo, hi]`.
fn integer_source_admits(lower: &Integer, upper: &Integer, target: &ValueType) -> bool {
    match target {
        ValueType::Integer => true,
        ValueType::Int(to) => to.lower() <= lower && upper <= to.upper(),
        ValueType::Rational(to) => {
            let one = Integer::one();
            to.numerator().lower() <= lower
                && upper <= to.numerator().upper()
                && to.denominator().lower() <= &one
                && &one <= to.denominator().upper()
        }
        ValueType::Decimal(to) => {
            let shift = u64::from(to.min_scale());
            compare_shifted(lower, shift, to.lower()).is_ge()
                && compare_shifted(upper, shift, to.upper()).is_le()
        }
        ValueType::Boolean
        | ValueType::Float(_)
        | ValueType::Quantity(_)
        | ValueType::Text(_)
        | ValueType::Uuid
        | ValueType::Timestamp
        | ValueType::Enum(_)
        | ValueType::Option(_)
        | ValueType::Composite(_)
        | ValueType::Collection(_)
        | ValueType::Reference(_)
        | ValueType::Population(_) => false,
    }
}

#[cfg(test)]
mod native_type_tests {
    use super::*;
    use ix_trace_rs::trace;
    use quire_exact::{Timestamp, Uuid};

    /// Native kinds are valid declared leaves and admit only their own values.
    #[trace("TC-910", "FR-111-AC-1", "FR-370-AC-1")]
    #[test]
    fn native_type_admission_preserves_distinct_kinds() {
        let types = TypeEnvironment::new([], []).expect("empty environment");
        let uuid = Value::Uuid(
            Uuid::from_canonical_text("00112233-4455-6677-8899-aabbccddeeff")
                .expect("canonical UUID"),
        );
        let timestamp =
            Value::Timestamp(Timestamp::from_canonical_text("1").expect("canonical timestamp"));

        let bounded_integer = ValueType::Int(quire_exact::IntegerInterval::spanning(
            Integer::zero(),
            Integer::one(),
        ));
        for value_type in [ValueType::Uuid, ValueType::Timestamp] {
            assert_eq!(types.check_type(&value_type), Ok(()));
            assert!(!types.contains_ieee(&value_type));
            assert!(!admits_equality_conversion(
                &bounded_integer,
                &value_type,
                &UnitScope::new(types.units()),
            ));
        }
        assert!(types.admits(&ValueType::Uuid, &uuid));
        assert!(types.admits(&ValueType::Timestamp, &timestamp));
        assert!(!types.admits(&ValueType::Uuid, &timestamp));
        assert!(!types.admits(&ValueType::Timestamp, &uuid));
        assert!(!types.admits(&ValueType::Uuid, &Value::Integer(Integer::one())));
        assert!(!types.admits(&ValueType::Timestamp, &Value::Integer(Integer::one())));
    }
}

/// The comparison value of one operand after its admitted conversion, with
/// quantity units read from `units`.
pub fn operand_value(
    operand: &EqualityOperand,
    value: &Value,
    units: &UnitScope<'_>,
    meter: &mut Meter,
) -> Result<Value, Stop> {
    // A `Reference<T>` operand's value carries its object's most specific
    // type, which is `T` or a type conforming to it (`lookup<T>` returns
    // the object as found): `ValueType::admits`'s exact type match would
    // refuse a real upcast, so a reference operand is checked by kind only.
    // The checker already proved the static types related.
    // The checker guarantees the value's type conforms to `T`; the plan then
    // compares only the identity triple, so the kind check is sufficient.
    let admitted = match (&operand.source, value) {
        (ValueType::Reference(_), value) => matches!(value, Value::Reference(_)),
        (source, value) => source.admits(value),
    };
    if !admitted {
        return Err(invariant(
            CheckedInvariantCause::EqualityOperandSourceNotAdmitted,
        ));
    }
    let Some(target) = &operand.target else {
        return Ok(value.clone());
    };
    let converted = match (&operand.source, target, value) {
        (source, target, value) if source == target => value.clone(),
        (_, ValueType::Integer | ValueType::Int(_), Value::Integer(_)) => value.clone(),
        (_, ValueType::Rational(_), Value::Integer(integer)) => {
            Value::Rational(Rational::from_integer(integer.clone()))
        }
        (_, ValueType::Rational(_), Value::Rational(_)) => value.clone(),
        (_, ValueType::Integer | ValueType::Int(_), Value::Rational(rational))
            if rational.is_integer() =>
        {
            Value::Integer(rational.numerator().clone())
        }
        (_, ValueType::Decimal(to), Value::Integer(integer)) => {
            integer_to_decimal(integer, to, meter)?
        }
        (_, ValueType::Decimal(to), Value::Rational(rational)) if rational.is_integer() => {
            integer_to_decimal(rational.numerator(), to, meter)?
        }
        (_, ValueType::Rational(_), Value::Decimal(decimal)) => {
            decimal_to_rational(decimal, meter)?
        }
        (_, ValueType::Decimal(to), Value::Decimal(decimal)) => {
            let result = outcome_into_stop(evaluate_decimal(
                DecimalOperation::Round(decimal),
                to,
                meter,
            ))?;
            Value::Decimal(result.value().clone())
        }
        (_, ValueType::Integer | ValueType::Int(_), Value::Decimal(decimal)) => {
            let rational = decimal.normalized().to_rational();
            if !rational.is_integer() {
                return Err(invariant(
                    CheckedInvariantCause::EqualityOperandNonIntegralDecimal,
                ));
            }
            Value::Integer(rational.numerator().clone())
        }
        (_, ValueType::Quantity(unit), Value::Quantity(quantity)) => {
            let (Some(source), Some(target)) = (units.resolve(quantity), units.get(*unit)) else {
                return Err(invariant(CheckedInvariantCause::EqualityUnitUnresolved));
            };
            let conversion = outcome_into_stop(
                convert_quantity(source, target, &QuantityTarget::Exact, meter).map_err(
                    |error| {
                        invariant(CheckedInvariantCause::EqualityQuantityConversionRejected {
                            cause: error.cause,
                        })
                    },
                )?,
            )?;
            match conversion.value() {
                ConvertedValue::Exact(exact) => {
                    Value::Quantity(Quantity::new(exact.clone(), *unit))
                }
                ConvertedValue::Decimal(_) | ConvertedValue::Integer { .. } => {
                    return Err(invariant(
                        CheckedInvariantCause::EqualityQuantityNonExactPlacement,
                    ))
                }
            }
        }
        _ => {
            return Err(invariant(
                CheckedInvariantCause::EqualityConversionShapeMismatch,
            ))
        }
    };
    if target.admits(&converted) {
        Ok(converted)
    } else {
        Err(invariant(
            CheckedInvariantCause::EqualityOperandTargetNotAdmitted,
        ))
    }
}

/// `Int[..]` or `Rational[..;d,1]` integer `n` into `Decimal[c1,c2;s1,s2]`,
/// retained as `(n × 10^s1, s1)`.
fn integer_to_decimal(
    value: &Integer,
    target: &DecimalType,
    meter: &mut Meter,
) -> Result<Value, Stop> {
    let scale = u64::from(target.min_scale());
    meter.charge(
        Charge::new(ChargePoint::DecimalOperands)
            .size(LimitKind::IntegerBits, value.magnitude_bits())
            .size(LimitKind::DecimalDigits, value.decimal_digits())
            .size(LimitKind::ValueOccurrences, 1),
    )?;
    let (bits, digits) = (sbits(value, scale), sdigits(value, scale));
    meter.charge(
        Charge::new(ChargePoint::DecimalScaleExpansion)
            .size(LimitKind::ScaleExpansion, scale)
            .exact_size(LimitKind::IntegerBits, bits.clone())
            .exact_size(LimitKind::DecimalDigits, digits.clone()),
    )?;
    meter.charge(
        Charge::new(ChargePoint::DecimalArithmetic)
            .exact_size(LimitKind::IntegerBits, bits.clone())
            .exact_size(LimitKind::DecimalDigits, digits.clone()),
    )?;
    // Retention upscales `n` at scale 0 by `k = s1`, sized before
    // `n × 10^s1` is materialized.
    meter.charge(
        Charge::new(ChargePoint::DecimalResultRetain)
            .size(LimitKind::ScaleExpansion, scale)
            .exact_size(LimitKind::IntegerBits, bits)
            .exact_size(LimitKind::DecimalDigits, digits)
            .size(LimitKind::ValueOccurrences, 1)
            .results(1),
    )?;
    let coefficient = value.mul(&Integer::power_of_ten(scale));
    Ok(Value::Decimal(Decimal::new(
        coefficient,
        target.min_scale(),
    )))
}

/// A `Decimal` `(c, s)` into an exact rational.
fn decimal_to_rational(value: &Decimal, meter: &mut Meter) -> Result<Value, Stop> {
    let representation = value.representation();
    let coefficient = representation.coefficient();
    let scale = u64::from(representation.scale());
    meter.charge(
        Charge::new(ChargePoint::DecimalOperands)
            .size(LimitKind::IntegerBits, coefficient.magnitude_bits())
            .size(LimitKind::DecimalDigits, coefficient.decimal_digits())
            .size(LimitKind::ValueOccurrences, 1),
    )?;
    meter.charge(
        Charge::new(ChargePoint::DecimalScaleExpansion)
            .size(LimitKind::ScaleExpansion, scale)
            .exact_size(LimitKind::IntegerBits, power_of_ten_bits(scale)),
    )?;
    meter.charge(
        Charge::new(ChargePoint::DecimalArithmetic)
            .exact_size(
                LimitKind::IntegerBits,
                Integer::from(coefficient.magnitude_bits()).max(power_of_ten_bits(scale)),
            )
            .exact_size(
                LimitKind::DecimalDigits,
                Integer::from(coefficient.decimal_digits())
                    .max(Integer::from(scale).add(&Integer::one())),
            ),
    )?;
    let rational = representation.to_rational();
    let maxparts = rational.max_part_bits();
    meter.charge(
        Charge::new(ChargePoint::DecimalResultRetain)
            .size(LimitKind::IntegerBits, maxparts)
            .size(LimitKind::ValueOccurrences, 1)
            .results(1),
    )?;
    Ok(Value::Rational(rational))
}

#[cfg(test)]
mod checked_invariant_tests {
    use super::*;
    use crate::enumeration::EnumDeclaration;
    use crate::unit::{DimensionNode, NominalDeclaration, UnitGraph, UnitNode};
    use ix_trace_rs::trace;
    use quire_exact::{EnumMember, ScalarLimits, UnitId, VariantId};

    fn meter() -> Meter {
        Meter::new(ScalarLimits {
            integer_bits: 1024,
            decimal_digits: 1024,
            scale_expansion: 1024,
            text_input_bytes: 1024,
            text_scalars: 1024,
            normalized_scalars: 1024,
            unit_edges: 1024,
            value_occurrences: 1024,
            work_units: 1024,
            result_units: 1024,
        })
    }

    fn checked(
        schedule: EqualitySchedule,
        left: EqualityOperand,
        right: EqualityOperand,
    ) -> CheckedEquality {
        CheckedEquality {
            operator: EqualityOperator::Equal,
            left,
            right,
            schedule,
            units: UnitTable::default(),
            enum_members: EnumMemberIndex::default(),
        }
    }

    fn refusal<T>(result: Result<T, Stop>, expected: CheckedInvariantCause) {
        assert!(matches!(
            result,
            Err(Stop::Refused(Refusal::CheckedInvariant { cause })) if cause == expected
        ));
    }

    /// Trace: FR-369-AC-9
    #[trace("TC-906", "FR-369-AC-9", "FR-369-AC-5")]
    #[test]
    fn deferred_result_names_failed_admission_and_preserves_prior_stop() {
        refusal(
            admitted(
                &ValueType::Boolean,
                Outcome::Completed(Value::Integer(Integer::one())),
            ),
            CheckedInvariantCause::DeferredResultNotAdmitted,
        );
        let Err(Stop::Refused(fault)) = admitted(
            &ValueType::Boolean,
            Outcome::Completed(Value::Integer(Integer::one())),
        ) else {
            panic!("failed admission must return the checked-invariant refusal");
        };
        assert_eq!(fault.code(), None);
        assert_eq!(fault.cause(), None);
        let ordinary = Refusal::IntegerOutOfDomain {
            target: Box::new(quire_exact::IntegerInterval::spanning(
                Integer::zero(),
                Integer::one(),
            )),
        };
        assert_eq!(ordinary.code(), Some("integer_out_of_domain"));
        assert_eq!(ordinary.cause(), Some("outside-domain"));
        assert!(matches!(
            admitted(
                &ValueType::Boolean,
                Outcome::Undefined(quire_exact::Undefined::DivisionByZero)
            ),
            Err(Stop::Undefined(quire_exact::Undefined::DivisionByZero))
        ));
        let mut exhausted = Meter::new(ScalarLimits {
            integer_bits: 1024,
            decimal_digits: 1024,
            scale_expansion: 1024,
            text_input_bytes: 1024,
            text_scalars: 1024,
            normalized_scalars: 1024,
            unit_edges: 1024,
            value_occurrences: 1024,
            work_units: 1024,
            result_units: 0,
        });
        let prior_stop = exhausted
            .charge(Charge::new(ChargePoint::CompositeResultRetain).results(1))
            .expect_err("zero result budget must deny retention");
        let Err(Stop::Incomplete(actual)) =
            admitted(&ValueType::Boolean, Outcome::Incomplete(prior_stop.clone()))
        else {
            panic!("a prior charge stop must survive admission");
        };
        assert_eq!(actual, prior_stop);
    }

    /// Trace: FR-369-AC-9
    #[trace("TC-906", "FR-369-AC-9")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the test supplies checked declaration identities without minting them in production"
    )]
    #[test]
    fn deferred_record_and_tuple_evaluation_preserve_admission_and_prior_refusal() {
        let record = NodeKey::from_digest([21; 32]);
        let tuple = NodeKey::from_digest([22; 32]);
        let environment = TypeEnvironment::new(
            [
                CompositeDeclaration::new(
                    record,
                    "R",
                    CompositeShape::Record(vec![FieldDeclaration::new(
                        "flag",
                        ValueType::Boolean,
                        Presence::Required,
                    )]),
                ),
                CompositeDeclaration::new(
                    tuple,
                    "T",
                    CompositeShape::Tuple(vec![ValueType::Boolean]),
                ),
            ],
            [],
        )
        .unwrap();

        let good =
            FieldExpression::Evaluate(Box::new(|_| Outcome::Completed(Value::Boolean(true))));
        let result = environment
            .evaluate_record(record, vec![("flag", good)], &mut meter())
            .unwrap();
        assert!(matches!(
            result,
            Outcome::Completed(Value::Composite(composite))
                if composite.declaration() == record
                    && matches!(composite.slots(), [FieldValue::Present(Value::Boolean(true))])
        ));
        let bad = FieldExpression::Evaluate(Box::new(|_| {
            Outcome::Completed(Value::Integer(Integer::one()))
        }));
        assert!(matches!(
            environment.evaluate_record(record, vec![("flag", bad)], &mut meter()),
            Ok(Outcome::Refused(Refusal::CheckedInvariant {
                cause: CheckedInvariantCause::DeferredResultNotAdmitted
            }))
        ));

        let result = environment
            .evaluate_tuple(
                tuple,
                vec![Box::new(|_| Outcome::Completed(Value::Boolean(true)))],
                &mut meter(),
            )
            .unwrap();
        assert!(matches!(
            result,
            Outcome::Completed(Value::Composite(composite))
                if composite.declaration() == tuple
                    && matches!(composite.slots(), [FieldValue::Present(Value::Boolean(true))])
        ));
        let prior = Refusal::IntegerOutOfDomain {
            target: Box::new(quire_exact::IntegerInterval::spanning(
                Integer::zero(),
                Integer::one(),
            )),
        };
        let result = environment
            .evaluate_tuple(
                tuple,
                vec![Box::new(|_| Outcome::Refused(prior.clone()))],
                &mut meter(),
            )
            .unwrap();
        assert!(matches!(result, Outcome::Refused(actual) if actual == prior));
    }

    /// Trace: FR-369-AC-9
    #[trace("TC-906", "FR-369-AC-9")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the test supplies a checked unit identity without minting one in production"
    )]
    #[test]
    fn equality_operand_failures_name_their_distinct_conditions() {
        let units = UnitTable::default();
        let scope = UnitScope::new(&units);
        let mut meter = meter();
        refusal(
            operand_value(
                &EqualityOperand::typed(ValueType::Boolean),
                &Value::Integer(Integer::one()),
                &scope,
                &mut meter,
            ),
            CheckedInvariantCause::EqualityOperandSourceNotAdmitted,
        );
        refusal(
            operand_value(
                &EqualityOperand::converted(ValueType::Boolean, ValueType::Integer),
                &Value::Boolean(true),
                &scope,
                &mut meter,
            ),
            CheckedInvariantCause::EqualityConversionShapeMismatch,
        );
        refusal(
            operand_value(
                &EqualityOperand::converted(
                    ValueType::Integer,
                    ValueType::Int(quire_exact::IntegerInterval::spanning(
                        Integer::zero(),
                        Integer::zero(),
                    )),
                ),
                &Value::Integer(Integer::one()),
                &scope,
                &mut meter,
            ),
            CheckedInvariantCause::EqualityOperandTargetNotAdmitted,
        );
        let decimal_type = ValueType::Decimal(
            DecimalType::new(
                Integer::zero(),
                Integer::from(10_i64),
                1,
                1,
                quire_exact::RoundingMode::Exact,
            )
            .unwrap(),
        );
        refusal(
            operand_value(
                &EqualityOperand::converted(decimal_type, ValueType::Integer),
                &Value::Decimal(Decimal::new(Integer::from(5_i64), 1)),
                &scope,
                &mut meter,
            ),
            CheckedInvariantCause::EqualityOperandNonIntegralDecimal,
        );
        let unresolved = quire_exact::UnitId::declared(NodeKey::from_digest([9; 32]));
        refusal(
            operand_value(
                &EqualityOperand::converted(
                    ValueType::Quantity(unresolved),
                    ValueType::Quantity(quire_exact::UnitId::declared(NodeKey::from_digest(
                        [10; 32],
                    ))),
                ),
                &Value::Quantity(Quantity::new(
                    Rational::from_integer(Integer::one()),
                    unresolved,
                )),
                &scope,
                &mut meter,
            ),
            CheckedInvariantCause::EqualityUnitUnresolved,
        );
    }

    /// Trace: FR-369-AC-9
    #[trace("TC-906", "FR-369-AC-9")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the test supplies a checked node identity without minting one in production"
    )]
    #[test]
    fn equality_schedule_distinguishes_shape_and_missing_identity() {
        let mut meter = meter();
        let mismatch = checked(
            EqualitySchedule::Text,
            EqualityOperand::typed(ValueType::Boolean),
            EqualityOperand::typed(ValueType::Boolean),
        );
        assert!(matches!(
            mismatch.evaluate(&Value::Boolean(true), &Value::Boolean(true), &mut meter),
            Outcome::Refused(Refusal::CheckedInvariant {
                cause: CheckedInvariantCause::EqualityScheduleMismatch
            })
        ));

        let unit = UnitId::declared(NodeKey::from_digest([1; 32]));
        let unresolved_unit = checked(
            EqualitySchedule::Quantity,
            EqualityOperand::typed(ValueType::Quantity(unit)),
            EqualityOperand::typed(ValueType::Quantity(unit)),
        );
        let quantity = Value::Quantity(Quantity::new(Rational::from_integer(Integer::one()), unit));
        assert!(matches!(
            unresolved_unit.evaluate(&quantity, &quantity, &mut meter),
            Outcome::Refused(Refusal::CheckedInvariant {
                cause: CheckedInvariantCause::EqualityUnitUnresolved
            })
        ));

        let variant = VariantId::from_digest([2; 32]);
        let enum_type = ValueType::Enum(EnumShape::new(true, [variant]));
        let unresolved_variant = checked(
            EqualitySchedule::Enum,
            EqualityOperand::typed(enum_type.clone()),
            EqualityOperand::typed(enum_type),
        );
        let member = Value::Enum(EnumMember::new(variant, 0));
        assert!(matches!(
            unresolved_variant.evaluate(&member, &member, &mut meter),
            Outcome::Refused(Refusal::CheckedInvariant {
                cause: CheckedInvariantCause::EqualityEnumVariantUnresolved
            })
        ));
    }

    /// Trace: FR-369-AC-9
    #[trace("TC-906", "FR-369-AC-9")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the test supplies checked enum identities without minting them in production"
    )]
    #[test]
    fn scheduled_comparator_preserves_its_ill_typed_cause() {
        let left = EnumDeclaration::new(NodeKey::from_digest([1; 32]), true, vec!["A".into()])
            .unwrap()
            .member("A", NodeKey::from_digest([3; 32]))
            .unwrap();
        let right = EnumDeclaration::new(NodeKey::from_digest([2; 32]), true, vec!["B".into()])
            .unwrap()
            .member("B", NodeKey::from_digest([4; 32]))
            .unwrap();
        let mut equality = checked(
            EqualitySchedule::Enum,
            EqualityOperand::typed(ValueType::Enum(EnumShape::new(true, [left.variant()]))),
            EqualityOperand::typed(ValueType::Enum(EnumShape::new(true, [right.variant()]))),
        );
        equality.enum_members.record(left.clone());
        equality.enum_members.record(right.clone());
        let mut meter = meter();
        assert!(matches!(
            equality.evaluate(
                &Value::Enum(EnumMember::new(left.variant(), 0)),
                &Value::Enum(EnumMember::new(right.variant(), 0)),
                &mut meter,
            ),
            Outcome::Refused(Refusal::CheckedInvariant {
                cause: CheckedInvariantCause::ScheduledComparisonRefused {
                    cause: IllTypedCause::DistinctEnumDeclarations
                }
            })
        ));
    }

    /// Trace: FR-369-AC-9
    #[trace("TC-906", "FR-369-AC-9")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the test supplies a checked enum identity without minting it in production"
    )]
    #[test]
    fn scheduled_comparison_keeps_an_earlier_charge_stop() {
        let member = EnumDeclaration::new(NodeKey::from_digest([31; 32]), true, vec!["A".into()])
            .unwrap()
            .member("A", NodeKey::from_digest([32; 32]))
            .unwrap();
        let mut equality = checked(
            EqualitySchedule::Enum,
            EqualityOperand::typed(ValueType::Enum(EnumShape::new(true, [member.variant()]))),
            EqualityOperand::typed(ValueType::Enum(EnumShape::new(true, [member.variant()]))),
        );
        equality.enum_members.record(member.clone());
        let value = Value::Enum(EnumMember::new(member.variant(), 0));
        let mut meter = meter().with_injected_denial(quire_exact::InjectedDenial {
            point: ChargePoint::EnumIdentityRead,
            occurrence: core::num::NonZeroU64::new(1).unwrap(),
        });
        assert!(matches!(
            equality.evaluate(&value, &value, &mut meter),
            Outcome::Incomplete(record) if record.charge_point == ChargePoint::EnumIdentityRead
        ));
    }

    /// Trace: FR-369-AC-9
    #[trace("TC-906", "FR-369-AC-9")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the test supplies checked unit identities without minting them in production"
    )]
    #[test]
    fn equality_quantity_conversion_preserves_its_ill_typed_cause() {
        let dimensions = BTreeMap::from([
            (
                NodeKey::from_digest([1; 32]),
                DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
            ),
            (
                NodeKey::from_digest([2; 32]),
                DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
            ),
        ]);
        let units = BTreeMap::from([
            (
                NodeKey::from_digest([3; 32]),
                UnitNode::checked(
                    [1; 32],
                    None,
                    Rational::from_integer(Integer::one()),
                    Rational::from_integer(Integer::zero()),
                    NominalDeclaration::default(),
                )
                .unwrap(),
            ),
            (
                NodeKey::from_digest([4; 32]),
                UnitNode::checked(
                    [2; 32],
                    None,
                    Rational::from_integer(Integer::one()),
                    Rational::from_integer(Integer::zero()),
                    NominalDeclaration::default(),
                )
                .unwrap(),
            ),
        ]);
        let graph = UnitGraph::from_checked_nodes(&dimensions, &units).unwrap();
        let table = UnitTable::declared(&graph);
        let scope = UnitScope::new(&table);
        let source = UnitId::declared(NodeKey::from_digest([3; 32]));
        let target = UnitId::declared(NodeKey::from_digest([4; 32]));
        refusal(
            operand_value(
                &EqualityOperand::converted(
                    ValueType::Quantity(source),
                    ValueType::Quantity(target),
                ),
                &Value::Quantity(Quantity::new(
                    Rational::from_integer(Integer::one()),
                    source,
                )),
                &scope,
                &mut meter(),
            ),
            CheckedInvariantCause::EqualityQuantityConversionRejected {
                cause: IllTypedCause::IncompatibleDimensions,
            },
        );
    }

    /// Trace: FR-369-AC-9
    #[trace("TC-906", "FR-369-AC-9")]
    #[allow(
        clippy::disallowed_methods,
        reason = "the test supplies checked unit identities without minting them in production"
    )]
    #[test]
    fn quantity_conversion_keeps_charge_stop_and_places_integer_successfully() {
        let dimension = NodeKey::from_digest([41; 32]);
        let source = UnitId::declared(NodeKey::from_digest([42; 32]));
        let target = UnitId::declared(NodeKey::from_digest([43; 32]));
        let dimensions = BTreeMap::from([(
            dimension,
            DimensionNode::checked(vec![], NominalDeclaration::default()).unwrap(),
        )]);
        let units = BTreeMap::from([
            (
                NodeKey::from_digest([42; 32]),
                UnitNode::checked(
                    [41; 32],
                    None,
                    Rational::from_integer(Integer::one()),
                    Rational::from_integer(Integer::zero()),
                    NominalDeclaration::default(),
                )
                .unwrap(),
            ),
            (
                NodeKey::from_digest([43; 32]),
                UnitNode::checked(
                    [41; 32],
                    Some([42; 32]),
                    Rational::from_integer(Integer::one()),
                    Rational::from_integer(Integer::zero()),
                    NominalDeclaration::default(),
                )
                .unwrap(),
            ),
        ]);
        let graph = UnitGraph::from_checked_nodes(&dimensions, &units).unwrap();
        let table = UnitTable::declared(&graph);
        let scope = UnitScope::new(&table);
        let quantity = Quantity::new(Rational::from_integer(Integer::one()), source);
        let mut denied = meter().with_injected_denial(quire_exact::InjectedDenial {
            point: ChargePoint::UnitIdentityRead,
            occurrence: core::num::NonZeroU64::new(1).unwrap(),
        });
        assert!(matches!(
            operand_value(
                &EqualityOperand::converted(
                    ValueType::Quantity(source),
                    ValueType::Quantity(target),
                ),
                &Value::Quantity(quantity.clone()),
                &scope,
                &mut denied,
            ),
            Err(Stop::Incomplete(record)) if record.charge_point == ChargePoint::UnitIdentityRead
        ));

        let conversion = convert_quantity(
            scope.resolve(&quantity).unwrap(),
            scope.get(target).unwrap(),
            &QuantityTarget::Integer {
                domain: quire_exact::IntegerInterval::spanning(
                    Integer::zero(),
                    Integer::from(10_i64),
                ),
                rounding: quire_exact::RoundingMode::Exact,
            },
            &mut meter(),
        )
        .unwrap();
        assert!(matches!(
            conversion,
            Outcome::Completed(value)
                if matches!(value.value(), ConvertedValue::Integer { value, loss: None }
                    if value.value() == &Integer::one())
        ));
    }
}

#[cfg(test)]
mod work_budget_tests {
    use super::WorkBudget;
    use super::{EnvironmentLimit, EnvironmentLimitKind};

    /// QSL FR-082: a denied charge names the cumulative total it would have
    /// reached: 3 spent, a budget of 4 and a charge of 2 report 5. The
    /// admitted spend is unchanged.
    #[test]
    fn a_denied_charge_reports_the_total_it_would_have_reached() {
        let cancel = quire_exact::Cancel::new();
        let mut budget = WorkBudget::new(4, Some(&cancel));
        assert_eq!(budget.charge(3), Ok(()));
        assert_eq!(
            budget.charge(2),
            Err(EnvironmentLimit::new(EnvironmentLimitKind::WorkUnits, 4, 5))
        );
        assert_eq!(budget.spent, 3);
        assert_eq!(budget.charge(1), Ok(()));
    }
}

#[cfg(test)]
mod ancestor_steps_tests {
    use alloc::format;

    use ix_trace_rs::trace;

    use super::*;

    #[allow(
        clippy::disallowed_methods,
        reason = "a test fixture needs an EffectiveId; production code mints none"
    )]
    fn id(byte: u8) -> EffectiveId {
        EffectiveId::from_digest([byte; 32])
    }

    fn object(byte: u8, supertypes: &[u8]) -> ObjectTypeDeclaration {
        ObjectTypeDeclaration::new(id(byte), format!("T{byte}"), vec![])
            .with_supertypes(supertypes.iter().map(|general| id(*general)).collect())
    }

    fn admit(types: Vec<ObjectTypeDeclaration>, ancestor_steps: u64) -> Admission<TypeEnvironment> {
        TypeEnvironment::bounded(
            [],
            types,
            TypeEnvironmentLimits {
                ancestor_steps,
                ..TypeEnvironmentLimits::default()
            },
        )
    }

    /// QSL FR-082-AC-6: `ancestor_steps` counts `supertypes` edges over a
    /// type's closure, never its types or its chain depth. A diamond
    /// `D -> B, C, A; B -> A; C -> A` has five edges over four types and no
    /// chain longer than two, so it admits at 5 and stops at 4. The refusal
    /// names the ceiling and, as its actual counter, the ceiling plus one:
    /// the edge the walk would stop at.
    #[trace("TC-220", "FR-082-AC-6")]
    #[test]
    fn ancestor_steps_counts_the_closure_edges_not_the_types_or_the_depth() {
        let diamond = || {
            vec![
                object(1, &[]),
                object(2, &[1]),
                object(3, &[1]),
                object(4, &[2, 3, 1]),
            ]
        };
        assert!(admit(diamond(), 5).is_ok());
        let Err(EnvironmentFailure::Limit(limit)) = admit(diamond(), 4) else {
            panic!("four edges are fewer than the diamond's five");
        };
        assert_eq!(limit.kind(), EnvironmentLimitKind::AncestorSteps);
        assert_eq!(limit.configured_bound(), 4);
        assert_eq!(limit.actual(), 5);
    }

    /// A chain of `n` edges admits at `n` and stops at `n - 1`.
    #[trace("TC-220", "FR-082-AC-6")]
    #[test]
    fn a_chain_of_n_edges_admits_at_n_and_stops_at_n_minus_one() {
        let chain = || {
            vec![
                object(1, &[]),
                object(2, &[1]),
                object(3, &[2]),
                object(4, &[3]),
            ]
        };
        assert!(admit(chain(), 3).is_ok());
        assert!(matches!(
            admit(chain(), 2),
            Err(EnvironmentFailure::Limit(limit)) if limit.actual() == 3
        ));
    }

    /// The default is NFR-012's 16777216 edges.
    #[trace("TC-220", "FR-082-AC-6")]
    #[test]
    fn the_default_ceiling_is_sixteen_million_edges() {
        assert_eq!(TypeEnvironmentLimits::default().ancestor_steps, 16_777_216);
    }
}
