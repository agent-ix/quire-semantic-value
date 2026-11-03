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
//!
//! Each object's slots are stored in the slot order of the environment the
//! closure was admitted under, beside that order's attribute names, so a
//! reader holding another environment finds an attribute by name rather than
//! by position (FR-106-AC-12).

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::declaration::{fill_slots, ConstructionRefusal, FieldRef, TypeEnvironment};
use crate::relationship_extent::{ManifestOutcome, Resolution};
use quire_exact::{EffectiveId, FieldValue, ObjectReference, UniverseId, Value};

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
    /// Each admitted object type's attribute names in slot order, held once
    /// per type: `names[t][i]` names slot `i` of every object of type `t`.
    names: BTreeMap<EffectiveId, Box<[String]>>,
}

/// One admitted object's attributes, each named as the admitting
/// environment declared it, in that environment's slot order.
#[derive(Clone, Copy, Debug)]
pub struct ObjectAttributes<'a> {
    names: &'a [String],
    slots: &'a [FieldValue],
}

impl<'a> ObjectAttributes<'a> {
    /// The slot of the attribute named `name`, or `None` when the admitting
    /// environment declared no such attribute for the object's type.
    pub fn get(self, name: &str) -> Option<&'a FieldValue> {
        self.iter()
            .find_map(|(held, slot)| (held == name).then_some(slot))
    }

    /// Every `(name, slot)` pair, in the admitting environment's slot order.
    /// An omitted `?` attribute is listed with an `Absent` slot.
    pub fn iter(self) -> impl Iterator<Item = (&'a str, &'a FieldValue)> + Clone {
        self.names.iter().map(String::as_str).zip(self.slots)
    }
}

impl ObjectClosure {
    /// Resolve a reference using held objects first, then exhaustive population evidence.
    pub fn resolve<'t, P: Ord + Clone, E>(
        &self,
        reference: &ObjectReference,
        types: &'t TypeEnvironment,
        manifest: &ManifestOutcome<'t, P, E>,
    ) -> Resolution {
        if self.contains(reference) {
            return Resolution::Present;
        }
        if !types.contains_object_type(reference.object_type()) {
            return Resolution::Undecidable;
        }
        if manifest.closed_coverage(types, reference.object_type()) {
            Resolution::Dangling
        } else {
            Resolution::Undecidable
        }
    }

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
        let mut admitted = BTreeMap::new();
        let mut names = BTreeMap::new();
        for (reference, attributes) in objects {
            let refuse = |cause| ObjectClosureRefusal {
                object: Box::new(reference.clone()),
                cause,
            };
            let Some(declared) = types.attributes(reference.object_type().clone()) else {
                return Err(refuse(ObjectClosureCause::UnknownObjectType));
            };
            let slots = fill_slots(types, declared, attributes)
                .map_err(|refusal| refuse(ObjectClosureCause::Attribute(refusal)))?;
            if admitted.contains_key(&reference) {
                return Err(refuse(ObjectClosureCause::DuplicateObject));
            }
            names
                .entry(reference.object_type().clone())
                .or_insert_with(|| {
                    declared
                        .iter()
                        .map(|attribute| String::from(attribute.field().name()))
                        .collect()
                });
            admitted.insert(reference, slots);
        }
        let closure = Self {
            objects: admitted,
            names,
        };
        let tolerated: BTreeSet<&ObjectReference> = tolerated_dangling.iter().collect();
        for (owner, slots) in &closure.objects {
            closure.check_closed(owner, slots, &tolerated)?;
        }
        Ok(closure)
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
            reference.universe() == &universe && reference.object().as_str() == object
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
    ///
    /// `types` resolves `field` to that attribute's name, and the slot is
    /// found by that name, so `types` need not be the environment the
    /// closure was admitted under: `None` when the admitting environment
    /// declared no attribute of that name for the object's type.
    pub fn attribute(
        &self,
        types: &TypeEnvironment,
        reference: &ObjectReference,
        field: &FieldRef,
    ) -> Option<&FieldValue> {
        let attribute = types
            .attributes(reference.object_type().clone())?
            .iter()
            .find(|attribute| attribute.stands_for(field))?;
        self.object(reference)?.get(attribute.field().name())
    }

    /// The admitted attributes of `reference`, or `None` when the closure
    /// holds no such object.
    pub fn object(&self, reference: &ObjectReference) -> Option<ObjectAttributes<'_>> {
        let slots = self.objects.get(reference)?;
        Some(self.attributes_of(reference, slots))
    }

    /// Every admitted object with its attributes, in reference order.
    pub fn iter(&self) -> impl Iterator<Item = (&ObjectReference, ObjectAttributes<'_>)> + Clone {
        self.objects
            .iter()
            .map(|(reference, slots)| (reference, self.attributes_of(reference, slots)))
    }

    fn attributes_of<'s>(
        &'s self,
        reference: &ObjectReference,
        slots: &'s [FieldValue],
    ) -> ObjectAttributes<'s> {
        // `new` records every admitted object's type; an unrecorded type
        // (unreachable) reads as no attribute names rather than panicking.
        let names = self
            .names
            .get(reference.object_type())
            .map_or(&[][..], |names| &names[..]);
        ObjectAttributes { names, slots }
    }

    fn check_closed(
        &self,
        owner: &ObjectReference,
        slots: &[FieldValue],
        tolerated: &BTreeSet<&ObjectReference>,
    ) -> Result<(), ObjectClosureRefusal> {
        let mut pending: Vec<&Value> = present(slots).collect();
        while let Some(value) = pending.pop() {
            match value {
                Value::Reference(reference)
                    if !self.objects.contains_key(reference) && !tolerated.contains(reference) =>
                {
                    return Err(ObjectClosureRefusal {
                        object: Box::new(owner.clone()),
                        cause: ObjectClosureCause::DanglingReference(Box::new(reference.clone())),
                    });
                }
                Value::Option(option) => pending.extend(option.payload()),
                Value::Composite(composite) => pending.extend(present(composite.slots())),
                Value::Collection(collection) => pending.extend(collection.elements()),
                Value::Union(union) => pending.extend(union.payload()),
                Value::Reference(_)
                | Value::Boolean(_)
                | Value::Integer(_)
                | Value::Rational(_)
                | Value::Decimal(_)
                | Value::Float(_)
                | Value::Quantity(_)
                | Value::Text(_)
                | Value::String(_)
                | Value::Uuid(_)
                | Value::Timestamp(_)
                | Value::Enum(_)
                | Value::Population(_) => {}
            }
        }
        Ok(())
    }
}

impl PartialEq for ObjectClosure {
    fn eq(&self, other: &Self) -> bool {
        self.names == other.names
            && self.objects.len() == other.objects.len()
            && self
                .objects
                .iter()
                .zip(other.objects.iter())
                .all(|((a_key, a), (b_key, b))| a_key == b_key && same_slots(a, b))
    }
}

impl Eq for ObjectClosure {}

enum IdentityPair<'a> {
    Slot(&'a FieldValue, &'a FieldValue),
    Value(&'a Value, &'a Value),
}

fn queue_slots<'a>(
    pending: &mut Vec<IdentityPair<'a>>,
    left: &'a [FieldValue],
    right: &'a [FieldValue],
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    pending.extend(
        left.iter()
            .zip(right)
            .map(|(a, b)| IdentityPair::Slot(a, b)),
    );
    true
}

fn queue_values<'a>(
    pending: &mut Vec<IdentityPair<'a>>,
    left: &'a [Value],
    right: &'a [Value],
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    pending.extend(
        left.iter()
            .zip(right)
            .map(|(a, b)| IdentityPair::Value(a, b)),
    );
    true
}

fn same_slots(left: &[FieldValue], right: &[FieldValue]) -> bool {
    let mut pending = Vec::new();
    if !queue_slots(&mut pending, left, right) {
        return false;
    }
    while let Some(pair) = pending.pop() {
        let equal = match pair {
            IdentityPair::Slot(a, b) => match (a, b) {
                (FieldValue::Absent, FieldValue::Absent) | (FieldValue::Null, FieldValue::Null) => {
                    true
                }
                (FieldValue::Present(a), FieldValue::Present(b)) => {
                    pending.push(IdentityPair::Value(a, b));
                    true
                }
                (FieldValue::Absent, FieldValue::Null | FieldValue::Present(_))
                | (FieldValue::Null, FieldValue::Absent | FieldValue::Present(_))
                | (FieldValue::Present(_), FieldValue::Absent | FieldValue::Null) => false,
            },
            IdentityPair::Value(a, b) => match (a, b) {
                (Value::Boolean(a), Value::Boolean(b)) => a == b,
                (Value::Integer(a), Value::Integer(b)) => a == b,
                (Value::Rational(a), Value::Rational(b)) => a == b,
                (Value::Decimal(a), Value::Decimal(b)) => a.representation() == b.representation(),
                (Value::Float(a), Value::Float(b)) => {
                    a.width() == b.width() && a.bits() == b.bits()
                }
                (Value::Quantity(a), Value::Quantity(b)) => a == b,
                (Value::Text(a), Value::Text(b)) => {
                    a.text_type() == b.text_type()
                        && a.payload() == b.payload()
                        && a.retained() == b.retained()
                }
                (Value::String(a), Value::String(b)) => a == b,
                (Value::Uuid(a), Value::Uuid(b)) => a == b,
                (Value::Timestamp(a), Value::Timestamp(b)) => a == b,
                (Value::Enum(a), Value::Enum(b)) => a == b,
                (Value::Population(a), Value::Population(b)) => a == b,
                (Value::Option(a), Value::Option(b)) => {
                    a.payload_type() == b.payload_type()
                        && match (a.payload(), b.payload()) {
                            (None, None) => true,
                            (Some(a), Some(b)) => {
                                pending.push(IdentityPair::Value(a, b));
                                true
                            }
                            (None, Some(_)) | (Some(_), None) => false,
                        }
                }
                (Value::Composite(a), Value::Composite(b)) => {
                    a.declaration() == b.declaration()
                        && queue_slots(&mut pending, a.slots(), b.slots())
                }
                (Value::Union(a), Value::Union(b)) => {
                    a.member() == b.member() && queue_values(&mut pending, a.payload(), b.payload())
                }
                (Value::Collection(a), Value::Collection(b)) => {
                    a.collection_type() == b.collection_type()
                        && queue_values(&mut pending, a.elements(), b.elements())
                }
                (Value::Reference(a), Value::Reference(b)) => a == b,
                (Value::Boolean(_), _)
                | (Value::Integer(_), _)
                | (Value::Rational(_), _)
                | (Value::Decimal(_), _)
                | (Value::Float(_), _)
                | (Value::Quantity(_), _)
                | (Value::Text(_), _)
                | (Value::String(_), _)
                | (Value::Uuid(_), _)
                | (Value::Timestamp(_), _)
                | (Value::Enum(_), _)
                | (Value::Population(_), _)
                | (Value::Option(_), _)
                | (Value::Composite(_), _)
                | (Value::Union(_), _)
                | (Value::Collection(_), _)
                | (Value::Reference(_), _) => false,
            },
        };
        if !equal {
            return false;
        }
    }
    true
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
    use crate::declaration::{
        Component, ConstructionCause, FieldDeclaration, ObjectTypeDeclaration,
    };
    use alloc::vec;
    use ix_trace_rs::trace;
    use quire_exact::{EffectiveId, ObjectId, Presence, Timestamp, Uuid, ValueType};

    #[allow(
        clippy::disallowed_methods,
        reason = "a test fixture needs an EffectiveId; production code mints none"
    )]
    fn object_type(byte: u8) -> EffectiveId {
        crate::test_identity::effective(u64::from(byte))
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
            crate::test_identity::universe(9),
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
        let universe = crate::test_identity::universe(9);
        let a = reference(A, "k");
        let single = ObjectClosure::new(&types(), [object(&a)], &[]).unwrap();
        assert_eq!(single.find(universe.clone(), "k"), Some(&a));

        let b = reference(B, "k");
        let both = ObjectClosure::new(&types(), [object(&a), object(&b)], &[]).unwrap();
        assert!(both.contains(&a) && both.contains(&b));
        assert_eq!(both.find(universe.clone(), "k"), None);
    }

    /// Native leaves are admitted as their own attribute kinds and do not
    /// contribute a dangling reference to closure traversal.
    #[trace("TC-910", "FR-111-AC-2", "FR-370-AC-1")]
    #[test]
    fn native_attributes_admit_and_wrong_kinds_refuse() {
        let native_type = object_type(4);
        let types = TypeEnvironment::new(
            [],
            [ObjectTypeDeclaration::new(
                native_type.clone(),
                "Native",
                vec![
                    FieldDeclaration::new("u", ValueType::Uuid, Presence::Required),
                    FieldDeclaration::new("t", ValueType::Timestamp, Presence::Required),
                ],
            )],
        )
        .expect("native leaf attributes admit as declarations");
        let owner = reference(4, "native");
        let expected_uuid = Uuid::from_canonical_text("00112233-4455-6677-8899-aabbccddeeff")
            .expect("canonical UUID");
        let expected_timestamp = Timestamp::from_canonical_text("-1").expect("canonical timestamp");
        let uuid = Value::Uuid(expected_uuid);
        let timestamp = Value::Timestamp(expected_timestamp);
        let attributes = |u, t| vec![("u", FieldValue::Present(u)), ("t", FieldValue::Present(t))];
        let closure = ObjectClosure::new(
            &types,
            [(owner.clone(), attributes(uuid.clone(), timestamp.clone()))],
            &[],
        )
        .expect("native leaves close without referenced objects");
        assert!(matches!(
            closure.attribute(&types, &owner, &FieldRef::new(native_type.clone(), "u")),
            Some(FieldValue::Present(Value::Uuid(actual))) if *actual == expected_uuid
        ));
        assert!(matches!(
            closure.attribute(&types, &owner, &FieldRef::new(native_type.clone(), "t")),
            Some(FieldValue::Present(Value::Timestamp(actual))) if *actual == expected_timestamp
        ));

        for (u, t, field) in [
            (timestamp.clone(), timestamp.clone(), "u"),
            (uuid.clone(), uuid.clone(), "t"),
            (Value::Integer(1_i64.into()), timestamp.clone(), "u"),
            (uuid.clone(), Value::Integer(1_i64.into()), "t"),
        ] {
            assert_eq!(
                ObjectClosure::new(&types, [(owner.clone(), attributes(u, t))], &[])
                    .expect_err("wrong attribute kind must refuse"),
                ObjectClosureRefusal {
                    object: Box::new(owner.clone()),
                    cause: ObjectClosureCause::Attribute(ConstructionRefusal {
                        component: Component::Field(field.into()),
                        cause: ConstructionCause::TypeMismatch,
                    }),
                }
            );
        }
    }

    /// Object type `A` declared with integer attributes in `order`, an
    /// optional `note` last.
    fn ordered(order: [&str; 2]) -> TypeEnvironment {
        let int = |name| FieldDeclaration::new(name, ValueType::Integer, Presence::Required);
        TypeEnvironment::new(
            [],
            [ObjectTypeDeclaration::new(
                object_type(A),
                "A",
                vec![
                    int(order[0]),
                    int(order[1]),
                    FieldDeclaration::new("note", ValueType::Integer, Presence::Optional),
                ],
            )],
        )
        .expect("one object type admits")
    }

    fn int(value: i64) -> FieldValue {
        FieldValue::Present(Value::Integer(value.into()))
    }

    /// The integer a slot holds; `None` for an absent, null or non-integer
    /// slot (`FieldValue` has no `PartialEq`).
    fn held(slot: &FieldValue) -> Option<quire_exact::Integer> {
        match slot {
            FieldValue::Present(Value::Integer(integer)) => Some(integer.clone()),
            FieldValue::Present(_) | FieldValue::Absent | FieldValue::Null => None,
        }
    }

    fn integer(value: i64) -> Option<quire_exact::Integer> {
        Some(value.into())
    }

    /// TC-904 row 9: a closure admitted under `{b, a}` and read under
    /// `{a, b}` returns each same-typed attribute's own slot by name, and
    /// an attribute the admitting environment never declared reads `None`.
    #[trace("TC-904", "FR-106-AC-12")]
    #[test]
    fn attribute_reads_by_name_across_reordered_environments() {
        let admitting = ordered(["b", "a"]);
        let reading = ordered(["a", "b"]);
        let owner = reference(A, "o");
        let closure = ObjectClosure::new(
            &admitting,
            [(owner.clone(), vec![("a", int(1)), ("b", int(2))])],
            &[],
        )
        .expect("one object closes");
        let field = |name| FieldRef::new(object_type(A), name);
        let read = |types: &TypeEnvironment, target: &ObjectReference, name| {
            closure.attribute(types, target, &field(name)).map(held)
        };
        assert_eq!(read(&reading, &owner, "a"), Some(integer(1)));
        assert_eq!(read(&reading, &owner, "b"), Some(integer(2)));

        let renamed = ordered(["a", "c"]);
        assert_eq!(read(&renamed, &owner, "c"), None);
        assert_eq!(read(&reading, &reference(A, "none"), "a"), None);

        // Equal slot vectors under reordered names are different objects:
        // `a = 2, b = 1` under `{a, b}` stores `[2, 1]`, as `closure` does.
        let swapped = ObjectClosure::new(
            &reading,
            [(owner.clone(), vec![("a", int(2)), ("b", int(1))])],
            &[],
        )
        .expect("one object closes");
        assert_ne!(closure, swapped);
        let rebuilt = ObjectClosure::new(
            &admitting,
            [(owner.clone(), vec![("b", int(2)), ("a", int(1))])],
            &[],
        )
        .expect("one object closes");
        assert_eq!(closure, rebuilt);
    }

    /// TC-904 row 11: attributes of different types, more than two of them
    /// and inherited from two supertypes whose order differs between the
    /// admitting and the reading environment are each read by name.
    #[trace("TC-904", "FR-106-AC-12")]
    #[test]
    fn attribute_reads_mixed_and_inherited_attributes_by_name() {
        const P: u8 = 5;
        const Q: u8 = 6;
        let field = |name, value_type| FieldDeclaration::new(name, value_type, Presence::Required);
        let environment = |own: [(&'static str, ValueType); 3], supertypes: [u8; 2]| {
            TypeEnvironment::new(
                [],
                [
                    ObjectTypeDeclaration::new(
                        object_type(A),
                        "A",
                        own.into_iter().map(|(name, ty)| field(name, ty)).collect(),
                    )
                    .with_supertypes(supertypes.into_iter().map(object_type).collect()),
                    ObjectTypeDeclaration::new(
                        object_type(P),
                        "P",
                        vec![field("p", ValueType::Integer)],
                    ),
                    ObjectTypeDeclaration::new(
                        object_type(Q),
                        "Q",
                        vec![field("q", ValueType::Integer)],
                    ),
                ],
            )
            .expect("an object type with two supertypes admits")
        };
        let admitting = environment(
            [
                ("b", ValueType::Integer),
                ("a", ValueType::Boolean),
                ("c", ValueType::Integer),
            ],
            [P, Q],
        );
        let reading = environment(
            [
                ("c", ValueType::Integer),
                ("a", ValueType::Boolean),
                ("b", ValueType::Integer),
            ],
            [Q, P],
        );
        let owner = reference(A, "o");
        let closure = ObjectClosure::new(
            &admitting,
            [(
                owner.clone(),
                vec![
                    ("a", FieldValue::Present(Value::Boolean(true))),
                    ("b", int(1)),
                    ("c", int(3)),
                    ("p", int(4)),
                    ("q", int(5)),
                ],
            )],
            &[],
        )
        .expect("one object closes");
        let read = |owner_type, name| {
            closure
                .attribute(
                    &reading,
                    &owner,
                    &FieldRef::new(object_type(owner_type), name),
                )
                .map(held)
        };
        assert_eq!(read(A, "b"), Some(integer(1)));
        assert_eq!(read(A, "c"), Some(integer(3)));
        assert_eq!(read(P, "p"), Some(integer(4)));
        assert_eq!(read(Q, "q"), Some(integer(5)));
        assert!(matches!(
            closure.attribute(&reading, &owner, &FieldRef::new(object_type(A), "a")),
            Some(FieldValue::Present(Value::Boolean(true)))
        ));
    }

    /// TC-904 row 10: `object` and `iter` expose each object's `(name,
    /// slot)` pairs in the admitting slot order, an omitted optional
    /// attribute as `Absent`; `get` finds by name and misses an undeclared
    /// name.
    #[trace("TC-904", "FR-106-AC-12")]
    #[test]
    fn objects_iterate_as_named_slots() {
        let types = ordered(["b", "a"]);
        let first = reference(A, "1");
        let second = reference(A, "2");
        let closure = ObjectClosure::new(
            &types,
            [
                (
                    second.clone(),
                    vec![("a", int(3)), ("b", int(4)), ("note", int(5))],
                ),
                (first.clone(), vec![("a", int(1)), ("b", int(2))]),
            ],
            &[],
        )
        .expect("two objects close");

        let listed: Vec<_> = closure
            .iter()
            .map(|(reference, attributes)| {
                let named: Vec<_> = attributes
                    .iter()
                    .map(|(name, slot)| (name, held(slot)))
                    .collect();
                (reference.clone(), named)
            })
            .collect();
        assert_eq!(
            listed,
            vec![
                (
                    first.clone(),
                    vec![("b", integer(2)), ("a", integer(1)), ("note", None)]
                ),
                (
                    second.clone(),
                    vec![("b", integer(4)), ("a", integer(3)), ("note", integer(5))]
                ),
            ]
        );
        let first_attributes = closure.object(&first).expect("admitted object");
        assert!(matches!(
            first_attributes.get("note"),
            Some(FieldValue::Absent)
        ));

        let attributes = closure.object(&second).expect("admitted object");
        assert_eq!(attributes.get("note").map(held), Some(integer(5)));
        assert!(attributes.get("missing").is_none());
        assert!(closure.object(&reference(A, "none")).is_none());
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "a test fixture needs a NodeKey; production code mints none"
    )]
    fn node_key(byte: u8) -> quire_exact::NodeKey {
        crate::test_identity::node(u64::from(byte))
    }

    /// A union value's payload is traversed like a composite's slots: a
    /// reference inside it must name a closed object, and a nested union is
    /// walked too.
    #[trace("TC-904", "FR-106-AC-11")]
    #[test]
    fn union_payload_references_are_closed_over() {
        use quire_exact::{UnionMember, UnionValue};
        let types = types();
        let present_ref = reference(A, "present");
        let missing = reference(B, "missing");
        let closure =
            ObjectClosure::new(&types, [object(&present_ref)], &[]).expect("one object closes");
        let member = UnionMember::from_admitted(node_key(7), "m").expect("fixture union member");
        let union = |payload| UnionValue::from_admitted(member.clone(), payload);
        let slots = |value| vec![FieldValue::Present(value)];
        let owner = reference(A, "owner");
        let tolerated = BTreeSet::new();

        let closed = union(vec![
            Value::Integer(1_i64.into()),
            Value::Reference(present_ref.clone()),
        ]);
        assert_eq!(
            closure.check_closed(&owner, &slots(closed), &tolerated),
            Ok(())
        );

        let nested = union(vec![union(vec![Value::Reference(missing.clone())])]);
        assert_eq!(
            closure.check_closed(&owner, &slots(nested), &tolerated),
            Err(ObjectClosureRefusal {
                object: Box::new(owner.clone()),
                cause: ObjectClosureCause::DanglingReference(Box::new(missing.clone())),
            })
        );

        let tolerating: BTreeSet<&ObjectReference> = [&missing].into_iter().collect();
        let dangling = union(vec![Value::Reference(missing.clone())]);
        assert_eq!(
            closure.check_closed(&owner, &slots(dangling), &tolerating),
            Ok(())
        );
    }

    /// A dangling reference is refused through every carrier: an `Option`
    /// payload, a composite slot and a collection element.
    #[trace("TC-904", "FR-106-AC-11")]
    #[test]
    fn dangling_references_are_refused_through_option_collection_and_composite() {
        use quire_exact::{
            from_admitted, from_admitted_slots, CollectionKind, CollectionType, OptionValue,
        };
        let types = types();
        let closure = ObjectClosure::new(&types, [object(&reference(A, "present"))], &[])
            .expect("one object closes");
        let missing = reference(B, "missing");
        let owner = reference(A, "owner");
        let held = Value::Reference(missing.clone());
        let ref_type = ValueType::Reference(object_type(B));
        let option = OptionValue::present(ref_type.clone(), held.clone()).expect("option");
        let collection = from_admitted(
            CollectionType::new(CollectionKind::Sequence, ref_type, None),
            vec![held.clone()],
        );
        let composite = from_admitted_slots(
            node_key(5),
            vec![FieldValue::Present(held)].into_boxed_slice(),
        );
        for carrier in [option, collection, composite] {
            assert_eq!(
                closure.check_closed(
                    &owner,
                    &[FieldValue::Present(carrier.clone())],
                    &BTreeSet::new()
                ),
                Err(ObjectClosureRefusal {
                    object: Box::new(owner.clone()),
                    cause: ObjectClosureCause::DanglingReference(Box::new(missing.clone())),
                })
            );
            let tolerated: BTreeSet<&ObjectReference> = [&missing].into_iter().collect();
            assert_eq!(
                closure.check_closed(&owner, &[FieldValue::Present(carrier)], &tolerated),
                Ok(())
            );
        }
    }
}
