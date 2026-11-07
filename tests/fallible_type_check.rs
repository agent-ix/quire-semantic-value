// SPDX-License-Identifier: AGPL-3.0-or-later
//! External checking under real allocator denial in its own integration binary.
mod common;

use ix_trace_rs::trace;
use quire_exact::{
    CollectionKind, CollectionType, FloatType, IeeeWidth, IllTyped, IllTypedCause, ValueType,
};
use quire_semantic_value::declaration::{
    CompositeDeclaration, CompositeShape, StorageRequest, StorageUnit, TypeCheckError,
    TypeEnvironment,
};

#[global_allocator]
static ALLOCATOR: qsv_failing_allocator::FailingAllocator = qsv_failing_allocator::FailingAllocator;

fn collection(kind: CollectionKind, element: ValueType) -> ValueType {
    ValueType::collection(CollectionType::new(kind, element, None))
}

fn refused(cause: IllTypedCause) -> Result<(), TypeCheckError> {
    Err(TypeCheckError::Refused(IllTyped { cause }))
}

#[trace("TC-929", "FR-132-AC-2", "FR-132-AC-4")]
#[test]
fn real_denials_cover_root_ieee_pending_growth_and_visited_then_retry() {
    let key = common::node(1);
    let environment = TypeEnvironment::new(
        [CompositeDeclaration::new(
            key.clone(),
            "Three",
            CompositeShape::Tuple(vec![
                ValueType::Boolean,
                ValueType::Integer,
                ValueType::Float(FloatType::exact(IeeeWidth::Binary64)),
            ]),
        )],
        [],
    )
    .unwrap();
    let original_environment = environment.clone();
    let ty = collection(CollectionKind::Set, ValueType::Composite(key));
    let original_type = ty.clone();
    let oracle = refused(IllTypedCause::OperatorIneligible);
    assert_eq!(environment.try_check_type(&ty), oracle);
    // Exact reservations allocate: root pointer, IEEE pending pointer, visited
    // (key pointer, unit), then pending grows from one to two to three pointers.
    let pointer = core::mem::size_of::<&ValueType>();
    let visited = core::mem::size_of::<(&quire_exact::NodeKey, ())>();
    for (successful, expected_bytes) in [pointer, pointer, visited, 2 * pointer, 3 * pointer]
        .into_iter()
        .enumerate()
    {
        qsv_failing_allocator::arm(successful);
        let result = environment.try_check_type(&ty);
        let denied = qsv_failing_allocator::disarm();
        assert_eq!(denied, Some(expected_bytes), "boundary {successful}");
        assert_eq!(
            result,
            Err(TypeCheckError::Allocation(StorageRequest::new(
                1,
                StorageUnit::AdditionalElements,
            ))),
            "boundary {successful}",
        );
        assert_eq!(environment, original_environment);
        assert_eq!(ty, original_type);
        assert_eq!(environment.try_check_type(&ty), oracle);
    }
    // Success and unknown-key refusal both recover from the root denial; the
    // latter must not clone the unknown structural identity for its error.
    for (ty, oracle) in [
        (ValueType::Integer, Ok(())),
        (
            ValueType::Composite(common::node(99)),
            refused(IllTypedCause::TypeMismatch),
        ),
    ] {
        qsv_failing_allocator::arm(0);
        let result = environment.try_check_type(&ty);
        let denied = qsv_failing_allocator::disarm();
        assert_eq!(denied, Some(pointer));
        assert_eq!(
            result,
            Err(TypeCheckError::Allocation(StorageRequest::new(
                1,
                StorageUnit::AdditionalElements
            )))
        );
        assert_eq!(environment.try_check_type(&ty), oracle);
        assert_eq!(environment, original_environment);
    }
}

#[trace("TC-929", "FR-132-AC-1")]
#[test]
fn external_semantics_preserve_nested_traversal_and_refusal_priority() {
    let environment = TypeEnvironment::new([], []).unwrap();
    for ty in [
        ValueType::Boolean,
        ValueType::Integer,
        ValueType::String,
        ValueType::option(ValueType::option(ValueType::Integer)),
        collection(
            CollectionKind::Sequence,
            ValueType::Float(FloatType::exact(IeeeWidth::Binary64)),
        ),
    ] {
        assert_eq!(environment.try_check_type(&ty), Ok(()));
    }
    for ty in [
        ValueType::Composite(common::node(9)),
        ValueType::Reference(common::effective(9)),
    ] {
        assert_eq!(
            environment.try_check_type(&ty),
            refused(IllTypedCause::TypeMismatch)
        );
    }
    for ty in [
        ValueType::Population(None),
        ValueType::option(ValueType::Population(None)),
    ] {
        assert_eq!(
            environment.try_check_type(&ty),
            refused(IllTypedCause::OperatorIneligible)
        );
    }
    let unknown = ValueType::Composite(common::node(999));
    qsv_failing_allocator::arm(1);
    let unknown_result = environment.try_check_type(&unknown);
    let denied = qsv_failing_allocator::disarm();
    assert_eq!(
        denied, None,
        "unknown-key errors allocate no identity clone"
    );
    assert_eq!(unknown_result, refused(IllTypedCause::TypeMismatch));
    for kind in [
        CollectionKind::Set,
        CollectionKind::Bag,
        CollectionKind::OrderedSet,
    ] {
        let ieee = ValueType::option(ValueType::Float(FloatType::exact(IeeeWidth::Binary64)));
        assert_eq!(
            environment.try_check_type(&collection(kind, ieee)),
            refused(IllTypedCause::OperatorIneligible)
        );
        assert_eq!(
            environment.try_check_type(&collection(kind, ValueType::Composite(common::node(9)))),
            refused(IllTypedCause::TypeMismatch)
        );
    }
}

#[trace("TC-929", "FR-132-AC-1")]
#[test]
fn ieee_discovery_preserves_record_tuple_union_and_cycle_behavior() {
    use quire_exact::{Identifier, MapType, Presence};
    use quire_semantic_value::declaration::{
        FieldDeclaration, ObjectTypeDeclaration, UnionMemberDeclaration,
    };
    let float = ValueType::Float(FloatType::exact(IeeeWidth::Binary64));
    let environment = TypeEnvironment::new(
        [
            CompositeDeclaration::new(
                common::node(1),
                "Record",
                CompositeShape::Record(vec![FieldDeclaration::new(
                    "value",
                    float.clone(),
                    Presence::Required,
                )]),
            ),
            CompositeDeclaration::new(
                common::node(2),
                "Tuple",
                CompositeShape::Tuple(vec![ValueType::Composite(common::node(1))]),
            ),
            CompositeDeclaration::new(
                common::node(3),
                "Union",
                CompositeShape::Union(vec![
                    UnionMemberDeclaration::resolved(
                        Identifier::new("Leaf").unwrap(),
                        vec![ValueType::Composite(common::node(2))],
                    ),
                    UnionMemberDeclaration::resolved(
                        Identifier::new("Next").unwrap(),
                        vec![ValueType::option(ValueType::Composite(common::node(3)))],
                    ),
                ]),
            ),
            CompositeDeclaration::new(
                common::node(4),
                "Cycle",
                CompositeShape::Union(vec![UnionMemberDeclaration::resolved(
                    Identifier::new("Next").unwrap(),
                    vec![ValueType::option(ValueType::Composite(common::node(4)))],
                )]),
            ),
        ],
        [ObjectTypeDeclaration::new(
            common::effective(1),
            "Object",
            vec![],
        )],
    )
    .unwrap();
    assert_eq!(
        environment.try_check_type(&ValueType::Reference(common::effective(1))),
        Ok(())
    );
    for number in 1..=4 {
        let composite = ValueType::Composite(common::node(number));
        assert_eq!(environment.try_check_type(&composite), Ok(()));
        let expected = if number == 4 {
            Ok(())
        } else {
            refused(IllTypedCause::OperatorIneligible)
        };
        for kind in [
            CollectionKind::Set,
            CollectionKind::Bag,
            CollectionKind::OrderedSet,
        ] {
            assert_eq!(
                environment.try_check_type(&collection(kind, composite.clone())),
                expected
            );
        }
    }
    let map = ValueType::map(MapType::new(ValueType::option(float), None));
    assert_eq!(environment.try_check_type(&map), Ok(()));
    assert_eq!(
        environment.try_check_type(&collection(CollectionKind::Set, map)),
        refused(IllTypedCause::OperatorIneligible)
    );
}
