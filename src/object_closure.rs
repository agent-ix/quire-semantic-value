// SPDX-License-Identifier: AGPL-3.0-or-later
//! The core object closure of an object environment over the kernel
//! [`ObjectReference`] (QSpec FR-143-AC-3 and AC-9). Its admission rules are
//! AC-10 of this repository's `FR-106`.
//!
//! A reference is terminal: its identity is the snapshot-supplied
//! QSpec FR-009/FR-204 triple (universe, object-type declaration identity, object
//! identity), and equality never inspects the referenced state. No source
//! form creates one. Cycles between objects are representable only through
//! references resolved in an [`ObjectClosure`].
//!
//! The closure names only kernel ids and this crate's values, checked against
//! this crate's [`TypeEnvironment`], so it sits beside `containment`. QSL FR-089's
//! `PopulationId` to population-binding correspondence is not here: the binding
//! is the caller's model's, so the caller holds that map beside the closure it
//! builds.

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::declaration::{
    fill_slots_walk, unmetered, ConstructionRefusal, EnvironmentLimit, FieldRef, TypeEnvironment,
    WorkBudget,
};
use quire_exact::{FieldValue, ObjectReference, UniverseId, Value};

/// Why an object closure does not close.
///
/// `object` is boxed: `ObjectReference` grew past a fixed-size 32-byte
/// `UniverseId`, which pushed this refusal's stack
/// size over `clippy::result_large_err`'s threshold on the cold refusal
/// path.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("object closure refused at {object:?}: {cause:?}")]
pub struct ObjectClosureRefusal {
    /// The object where the refusal originates.
    pub object: Box<ObjectReference>,
    /// The typed cause.
    pub cause: ObjectClosureCause,
}

/// The typed cause of an [`ObjectClosureRefusal`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectClosureCause {
    /// Two objects share one identity triple.
    DuplicateObject,
    /// The object type is not a model object type of the environment.
    UnknownObjectType,
    /// An attribute does not match its declaration.
    Attribute(ConstructionRefusal),
    /// A contained reference names no object of the closure.
    DanglingReference(Box<ObjectReference>),
}

/// A closed set of objects: every reference held by any attribute resolves
/// to an object of the closure.
#[derive(Clone, Debug, Default)]
pub struct ObjectClosure {
    objects: BTreeMap<ObjectReference, Box<[FieldValue]>>,
}

impl ObjectClosure {
    /// Admit `objects` as `(reference, attributes)` pairs against the model
    /// object types of `types`. An omitted `?` attribute is `absent`.
    ///
    /// Every reference any attribute holds must name an object of the
    /// closure, except a reference in `tolerated_dangling`: the exact
    /// targets a caller has already decided may dangle. QSL FR-106 admission
    /// passes the references its check 8 skipped, because they name an
    /// incomplete population nothing requires; every other caller passes
    /// `&[]`.
    pub fn new<'n>(
        types: &TypeEnvironment,
        objects: impl IntoIterator<Item = (ObjectReference, Vec<(&'n str, FieldValue)>)>,
        tolerated_dangling: &[ObjectReference],
    ) -> Result<Self, ObjectClosureRefusal> {
        unmetered(Self::new_walk(
            types,
            objects,
            tolerated_dangling,
            &mut |_| Ok(()),
        ))
    }

    /// Admit objects, attribute types and reference closure with one shared budget.
    /// The outer result preserves named work stops and caller cancellation; the
    /// inner result preserves the existing object/attribute refusal and locus.
    pub fn new_with_budget<'n>(
        types: &TypeEnvironment,
        objects: impl IntoIterator<Item = (ObjectReference, Vec<(&'n str, FieldValue)>)>,
        tolerated_dangling: &[ObjectReference],
        budget: &mut WorkBudget,
    ) -> Result<Result<Self, ObjectClosureRefusal>, EnvironmentLimit> {
        Self::new_walk(types, objects, tolerated_dangling, &mut |units| {
            budget.charge(units)
        })
    }

    fn new_walk<'n, E>(
        types: &TypeEnvironment,
        objects: impl IntoIterator<Item = (ObjectReference, Vec<(&'n str, FieldValue)>)>,
        tolerated_dangling: &[ObjectReference],
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Result<Self, ObjectClosureRefusal>, E> {
        let mut admitted = BTreeMap::new();
        for (reference, attributes) in objects {
            charge(1)?;
            let refuse = |cause| ObjectClosureRefusal {
                object: Box::new(reference.clone()),
                cause,
            };
            let Some(declared) = types.attributes(reference.object_type()) else {
                return Ok(Err(refuse(ObjectClosureCause::UnknownObjectType)));
            };
            let slots = match fill_slots_walk(types, declared, attributes, charge)? {
                Ok(slots) => slots,
                Err(refusal) => return Ok(Err(refuse(ObjectClosureCause::Attribute(refusal)))),
            };
            if admitted.contains_key(&reference) {
                return Ok(Err(refuse(ObjectClosureCause::DuplicateObject)));
            }
            admitted.insert(reference, slots);
        }
        let closure = Self { objects: admitted };
        charge(tolerated_dangling.len())?;
        let tolerated: BTreeSet<&ObjectReference> = tolerated_dangling.iter().collect();
        for (owner, slots) in &closure.objects {
            charge(1)?;
            if let Err(refusal) = closure.check_closed(owner, slots, &tolerated, charge)? {
                return Ok(Err(refusal));
            }
        }
        Ok(Ok(closure))
    }

    /// The closure's own reference whose universe is `universe` and
    /// declared key is `object`, whatever its most-specific type is (QSL FR-109:
    /// a `Function` selection's object argument names an object by
    /// population and key alone, with no declared type of its own to
    /// narrow the search). `None` when no admitted object matches, or more
    /// than one does (an object identity is unique within one universe, so
    /// more than one match is a broken admission invariant, not a real
    /// ambiguity).
    pub fn find(&self, universe: UniverseId, object: &str) -> Option<&ObjectReference> {
        let mut found = self.objects.keys().filter(|reference| {
            reference.universe() == universe && reference.object().as_str() == object
        });
        let first = found.next()?;
        match found.next() {
            None => Some(first),
            Some(_) => None,
        }
    }

    /// Whether `reference` names an object of the closure.
    pub fn contains(&self, reference: &ObjectReference) -> bool {
        self.objects.contains_key(reference)
    }

    /// The referenced object's slot for `field`, the field `deref(r).f`
    /// resolved to in `r`'s static type. The object's own type
    /// conforms to that static type, so its effective attribute set has
    /// exactly one attribute standing for `field`: `field` itself when
    /// inherited unchanged, or the field that redefines it.
    pub fn attribute(
        &self,
        types: &TypeEnvironment,
        reference: &ObjectReference,
        field: &FieldRef,
    ) -> Option<&FieldValue> {
        let position = types
            .attributes(reference.object_type())?
            .iter()
            .position(|attribute| attribute.stands_for(field))?;
        self.objects.get(reference)?.get(position)
    }

    fn check_closed<E>(
        &self,
        owner: &ObjectReference,
        slots: &[FieldValue],
        tolerated: &BTreeSet<&ObjectReference>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Result<(), ObjectClosureRefusal>, E> {
        charge(slots.len())?;
        let mut pending: Vec<&Value> = present(slots).collect();
        while let Some(value) = pending.pop() {
            charge(1)?;
            match value {
                Value::Reference(reference)
                    if !self.objects.contains_key(reference) && !tolerated.contains(reference) =>
                {
                    return Ok(Err(ObjectClosureRefusal {
                        object: Box::new(owner.clone()),
                        cause: ObjectClosureCause::DanglingReference(Box::new(reference.clone())),
                    }));
                }
                Value::Option(option) => {
                    charge(usize::from(option.payload().is_some()))?;
                    pending.extend(option.payload());
                }
                Value::Composite(composite) => {
                    charge(composite.slots().len())?;
                    pending.extend(present(composite.slots()));
                }
                Value::Union(union) => {
                    charge(union.payload().len())?;
                    pending.extend(union.payload());
                }
                Value::Collection(collection) => {
                    charge(collection.elements().len())?;
                    pending.extend(collection.elements());
                }
                Value::Reference(_)
                | Value::Boolean(_)
                | Value::Integer(_)
                | Value::Rational(_)
                | Value::Decimal(_)
                | Value::Float(_)
                | Value::Quantity(_)
                | Value::Text(_)
                | Value::Enum(_)
                | Value::Population(_) => {}
            }
        }
        Ok(Ok(()))
    }
}

fn present(slots: &[FieldValue]) -> impl Iterator<Item = &Value> {
    slots.iter().filter_map(|slot| match slot {
        FieldValue::Present(value) => Some(value),
        FieldValue::Absent | FieldValue::Null => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declaration::{FieldDeclaration, ObjectTypeDeclaration};
    use alloc::vec;
    use ix_trace_rs::trace;
    use quire_exact::{EffectiveId, ObjectId, Presence, ValueType};

    #[allow(
        clippy::disallowed_methods,
        reason = "a test fixture needs an EffectiveId; production code mints none"
    )]
    fn object_type(byte: u8) -> EffectiveId {
        EffectiveId::from_digest([byte; 32])
    }

    const A: u8 = 1;
    const B: u8 = 2;
    const UNDECLARED: u8 = 3;

    /// Model object types `A` and `B`, each with one required `Int`
    /// attribute `x`.
    fn types() -> TypeEnvironment {
        let declare = |byte, name| {
            ObjectTypeDeclaration::new(
                object_type(byte),
                name,
                vec![FieldDeclaration::new(
                    "x",
                    ValueType::Integer,
                    Presence::Required,
                )],
            )
        };
        TypeEnvironment::new([], [declare(A, "A"), declare(B, "B")])
            .expect("two object types admit")
    }

    fn reference(type_byte: u8, key: &str) -> ObjectReference {
        ObjectReference::new(
            UniverseId::from_digest([9; 32]),
            object_type(type_byte),
            ObjectId::new(key).expect("non-empty key"),
        )
    }

    fn object(reference: &ObjectReference) -> (ObjectReference, Vec<(&'static str, FieldValue)>) {
        (
            reference.clone(),
            vec![("x", FieldValue::Present(Value::Integer(1_i64.into())))],
        )
    }

    /// TC-904 row 1: a second object with an admitted identity triple
    /// refuses `DuplicateObject`, naming that triple.
    #[trace("TC-904", "FR-106-AC-10")]
    #[test]
    fn a_duplicate_identity_triple_refuses() {
        let a = reference(A, "a");
        assert_eq!(
            ObjectClosure::new(&types(), [object(&a), object(&a)], &[]).unwrap_err(),
            ObjectClosureRefusal {
                object: Box::new(a),
                cause: ObjectClosureCause::DuplicateObject,
            }
        );
    }

    /// TC-904 row 2: an object whose type names no model object type of the
    /// environment refuses `UnknownObjectType`.
    #[trace("TC-904", "FR-106-AC-10")]
    #[test]
    fn a_non_model_object_type_refuses() {
        let stray = reference(UNDECLARED, "s");
        assert_eq!(
            ObjectClosure::new(&types(), [object(&stray)], &[]).unwrap_err(),
            ObjectClosureRefusal {
                object: Box::new(stray),
                cause: ObjectClosureCause::UnknownObjectType,
            }
        );
    }

    /// TC-904 rows 3 and 4: `find` returns the one object a universe and
    /// key name, and none when two objects of different types share them.
    #[trace("TC-904", "FR-106-AC-10")]
    #[test]
    fn find_returns_none_for_an_ambiguous_key() {
        let universe = UniverseId::from_digest([9; 32]);
        let a = reference(A, "k");
        let single = ObjectClosure::new(&types(), [object(&a)], &[]).unwrap();
        assert_eq!(single.find(universe, "k"), Some(&a));

        let b = reference(B, "k");
        let both = ObjectClosure::new(&types(), [object(&a), object(&b)], &[]).unwrap();
        assert!(both.contains(&a) && both.contains(&b));
        assert_eq!(both.find(universe, "k"), None);
    }
}
