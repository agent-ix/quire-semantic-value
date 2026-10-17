// SPDX-License-Identifier: AGPL-3.0-or-later
//! `ModelTransition` through the public `model_transition` API (TC-920).

use std::cmp::Ordering;

use ix_trace_rs::trace;
use quire_canonical::{to_vec, Limits};
use quire_semantic_value::model_transition::{
    InvalidModelTransition, ModelTransition, TransitionCause, TransitionSite,
};
use quire_semantic_value::semantic_node::IdentityRefusal;

const BIG: Limits = Limits::new(1 << 26);
const INCREMENT: &str = r#"{"digest_domain":"sha256-jcs","node":"test/counters/Counter/increment","package":"test/counters"}"#;

fn text(value: &str) -> String {
    value.to_owned()
}

fn texts(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| text(value)).collect()
}

fn build(operation: &str, receiver: Option<&str>, arguments: &[&str]) -> ModelTransition {
    ModelTransition::new(text(operation), receiver.map(text), texts(arguments), BIG).unwrap()
}

fn refused(
    operation: &str,
    receiver: Option<&str>,
    arguments: &[&str],
    limits: Limits,
) -> InvalidModelTransition {
    ModelTransition::new(
        text(operation),
        receiver.map(text),
        texts(arguments),
        limits,
    )
    .unwrap_err()
}

fn node(node: &str) -> String {
    format!(r#"{{"node":"{node}","package":"p"}}"#)
}

/// The fixtures of TC-920 steps 5 and 6.
fn fixtures() -> Vec<ModelTransition> {
    let x = node("x");
    let xy = node("x/y");
    let other = node("other");
    let mut fixtures = vec![
        build(INCREMENT, Some(r#""c1""#), &["true"]),
        build(INCREMENT, Some(r#""c1""#), &["true"]),
        build(&other, Some(r#""c1""#), &["true"]),
        build(INCREMENT, Some(r#""c1""#), &["false"]),
        build(INCREMENT, Some(r#""c1""#), &["1", "2"]),
        build(INCREMENT, Some(r#""c1""#), &["2", "1"]),
        build(INCREMENT, Some(r#""c1""#), &["1"]),
        build(INCREMENT, Some(r#""c2""#), &["true"]),
        build(INCREMENT, None, &["true"]),
        build(INCREMENT, Some(r#""null""#), &["true"]),
        build(&x, Some(r#""c1""#), &["true"]),
        build(&xy, Some(r#""c1""#), &["true"]),
        build(INCREMENT, Some(r#""c1""#), &["[1]"]),
        build(INCREMENT, Some(r#""c1""#), &["[1,2]"]),
    ];
    // Receivers that are not strings, and strings that need escaping: the
    // receiver is embedded as the JSON value it is, and an absent one is `null`.
    for receiver in [
        "false",
        "true",
        "0",
        "1",
        "-1",
        "1.5",
        "[]",
        "[1]",
        "[1,2]",
        "[[1]]",
        "{}",
        r#"{"a":1}"#,
        r#"{"a":[true,{"b":null}]}"#,
        r#""a\"b""#,
        r#""a\\b""#,
        r#""a\nb""#,
        r#""é""#,
    ] {
        fixtures.push(build(INCREMENT, Some(receiver), &["true"]));
    }
    fixtures
}

fn bytes(value: &ModelTransition) -> Vec<u8> {
    value.canonical_bytes(BIG).unwrap()
}

/// Trace: FR-120-AC-1
#[trace("TC-920", "FR-120-AC-1")]
#[test]
fn new_retains_operation_receiver_and_arguments_verbatim() {
    let arguments = ["true", "[1,2]", r#"{"a":1}"#];
    let value = build(INCREMENT, Some(r#""c1""#), &arguments);
    assert_eq!(value.operation(), INCREMENT);
    assert_eq!(value.receiver(), Some(r#""c1""#));
    assert_eq!(value.arguments(), texts(&arguments).as_slice());

    let none = build(INCREMENT, None, &[]);
    assert_eq!(none.receiver(), None);
    assert!(none.arguments().is_empty());

    let many: Vec<String> = (0..64).map(|n| n.to_string()).collect();
    let sixty_four = ModelTransition::new(text(INCREMENT), None, many.clone(), BIG).unwrap();
    assert_eq!(sixty_four.arguments(), many.as_slice());
}

/// Trace: FR-120-AC-2
#[trace("TC-920", "FR-120-AC-2")]
#[test]
fn new_refuses_a_non_canonical_operation_or_receiver() {
    for bad in [
        "increment",
        r#"{"b":1,"a":2}"#,
        r#" {"a":1}"#,
        r#"{"a":1} {"a":2}"#,
    ] {
        let error = refused(bad, Some(r#""c1""#), &["true"], BIG);
        assert_eq!(error.site(), TransitionSite::Operation, "{bad}");
        assert_eq!(error.cause(), &TransitionCause::NotCanonicalJson, "{bad}");
    }
    for bad in ["c1", r#"{"b":1,"a":2}"#] {
        let error = refused(INCREMENT, Some(bad), &["true"], BIG);
        assert_eq!(error.site(), TransitionSite::Receiver, "{bad}");
        assert_eq!(error.cause(), &TransitionCause::NotCanonicalJson, "{bad}");
    }
}

/// Trace: FR-120-AC-3
#[trace("TC-920", "FR-120-AC-3")]
#[test]
fn new_checks_operation_then_receiver_then_arguments_in_order() {
    let bad = r#"{"b":1,"a":2}"#;
    let error = refused(INCREMENT, None, &["true", bad, "false"], BIG);
    assert_eq!(error.site(), TransitionSite::Argument(1));
    assert_eq!(error.cause(), &TransitionCause::NotCanonicalJson);

    let error = refused(INCREMENT, None, &["[1, 2]", bad], BIG);
    assert_eq!(error.site(), TransitionSite::Argument(0));

    let error = refused("op", Some("rcv"), &["arg"], BIG);
    assert_eq!(error.site(), TransitionSite::Operation);
    let error = refused(INCREMENT, Some("rcv"), &["arg"], BIG);
    assert_eq!(error.site(), TransitionSite::Receiver);
}

/// Trace: FR-120-AC-4
#[trace("TC-920", "FR-120-AC-4")]
#[test]
fn canonical_bytes_are_the_jcs_transition_object() {
    let expected = concat!(
        r#"{"arguments":[true],"operation":"{\"digest_domain\":\"sha256-jcs\","#,
        r#"\"node\":\"test/counters/Counter/increment\",\"package\":\"test/counters\"}","#,
        r#""receiver":"c1","type":"transition"}"#
    );
    let value = build(INCREMENT, Some(r#""c1""#), &["true"]);
    assert_eq!(bytes(&value), expected.as_bytes());

    let absent = build(INCREMENT, None, &["true"]);
    assert!(String::from_utf8(bytes(&absent))
        .unwrap()
        .contains(r#""receiver":null,"#));

    let empty = build(INCREMENT, Some(r#""c1""#), &[]);
    assert!(String::from_utf8(bytes(&empty))
        .unwrap()
        .starts_with(r#"{"arguments":[],"operation":"#));

    let ordered = build(INCREMENT, Some(r#""c1""#), &["1", "2"]);
    assert!(String::from_utf8(bytes(&ordered))
        .unwrap()
        .starts_with(r#"{"arguments":[1,2],"operation":"#));
}

/// Trace: FR-120-AC-5
#[trace("TC-920", "FR-120-AC-5")]
#[test]
fn equality_is_field_wise_and_agrees_with_the_identity_bytes() {
    let all = fixtures();
    for (left_index, left) in all.iter().enumerate() {
        for (right_index, right) in all.iter().enumerate() {
            let same = left_index == right_index
                || (left_index, right_index) == (0, 1)
                || (left_index, right_index) == (1, 0);
            assert_eq!(left == right, same, "{left_index} {right_index}");
            assert_eq!(
                bytes(left) == bytes(right),
                same,
                "{left_index} {right_index}"
            );
        }
    }
    let present = build(INCREMENT, Some(r#""c1""#), &["true"]);
    let other = build(INCREMENT, Some(r#""c2""#), &["true"]);
    let (present, other) = (bytes(&present), bytes(&other));
    let prefix = present
        .iter()
        .zip(&other)
        .take_while(|(a, b)| a == b)
        .count();
    let suffix = present
        .iter()
        .rev()
        .zip(other.iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let differing = &present[prefix..present.len() - suffix];
    assert_eq!(differing, b"1");
    let receiver_member = br#""receiver":"c"#;
    assert!(present
        .windows(receiver_member.len())
        .any(|window| window == receiver_member));
    assert!(prefix + suffix + 1 == present.len());
}

/// Trace: FR-120-AC-6
#[trace("TC-920", "FR-120-AC-6")]
#[test]
fn order_is_the_byte_order_not_the_field_order() {
    let (x, xy) = (node("x"), node("x/y"));
    let short = build(&x, Some(r#""c1""#), &["true"]);
    let long = build(&xy, Some(r#""c1""#), &["true"]);
    assert_eq!(short.cmp(&long), bytes(&short).cmp(&bytes(&long)));
    assert_eq!(short.cmp(&long), Ordering::Greater);
    assert_eq!(
        x.cmp(&xy),
        Ordering::Less,
        "raw strings order the other way"
    );

    let one = build(INCREMENT, Some(r#""c1""#), &["[1]"]);
    let one_two = build(INCREMENT, Some(r#""c1""#), &["[1,2]"]);
    assert_eq!(one.cmp(&one_two), bytes(&one).cmp(&bytes(&one_two)));
    assert_eq!(one_two.cmp(&one), Ordering::Less);
}

/// Trace: FR-120-AC-7
#[trace("TC-920", "FR-120-AC-7")]
#[test]
fn encode_writes_the_canonical_bytes_and_the_same_refusal() {
    for value in [
        build(INCREMENT, Some(r#""c1""#), &["true"]),
        build(INCREMENT, None, &[]),
        build(INCREMENT, Some(r#""c1""#), &["1", "2"]),
    ] {
        assert_eq!(to_vec(&value, BIG).unwrap(), bytes(&value));
        let short = Limits::new(u64::try_from(bytes(&value).len()).unwrap() - 1);
        let from_encode = IdentityRefusal::from(to_vec(&value, short).unwrap_err());
        let from_bytes = value.canonical_bytes(short).unwrap_err();
        assert_eq!(from_encode, from_bytes);
        assert!(matches!(
            from_bytes,
            IdentityRefusal::InputBytes { bound, required } if required > bound
        ));
    }
}

/// Trace: FR-120-AC-8
#[trace("TC-920", "FR-120-AC-8")]
#[test]
fn new_bounds_each_text_on_its_own() {
    let operation = node("a-long-operation-node-name");
    let receiver = r#""receiver-text-long""#;
    let argument = r#"["argument-one-is-the-long-one"]"#;
    let build_under = |limit: u64, op: &str, rcv: &str, args: &[&str]| {
        ModelTransition::new(text(op), Some(text(rcv)), texts(args), Limits::new(limit))
    };

    let error = build_under(10, &operation, "1", &["1"]).unwrap_err();
    assert_eq!(error.site(), TransitionSite::Operation);
    let &TransitionCause::InputBytes { bound, required } = error.cause() else {
        panic!("not an input-bytes cause");
    };
    assert_eq!(bound, 10);
    assert_eq!(required, u64::try_from(operation.len()).unwrap());

    let error = build_under(10, "1", receiver, &["1"]).unwrap_err();
    assert_eq!(error.site(), TransitionSite::Receiver);
    assert!(matches!(
        error.cause(),
        TransitionCause::InputBytes { bound: 10, required } if *required > 10
    ));

    let error = build_under(10, "1", "2", &["1", argument]).unwrap_err();
    assert_eq!(error.site(), TransitionSite::Argument(1));
    assert!(matches!(
        error.cause(),
        TransitionCause::InputBytes { bound: 10, required } if *required > 10
    ));

    let limit = 40;
    let (a, b, c) = (
        r#""a-text-of-about-20-by""#,
        r#""b-text-of-about-20-by""#,
        "1",
    );
    assert!(a.len() <= limit && b.len() <= limit && a.len() + b.len() > limit);
    let admitted = build_under(40, "1", a, &[b, c]).unwrap();
    assert_eq!(admitted.arguments().len(), 2);
}

/// Trace: FR-120-AC-12
#[trace("TC-920", "FR-120-AC-12")]
#[test]
fn a_null_receiver_is_refused_and_cmp_agrees_with_equality() {
    let error = refused(INCREMENT, Some("null"), &["true"], BIG);
    assert_eq!(error.site(), TransitionSite::Receiver);
    assert_eq!(error.cause(), &TransitionCause::NullReceiver);

    let all = fixtures();
    for left in &all {
        for right in &all {
            assert_eq!(left.cmp(right) == Ordering::Equal, left == right);
        }
    }
}

/// Trace: FR-120-AC-13
#[trace("TC-920", "FR-120-AC-13")]
#[test]
fn canonical_bytes_refuses_a_limit_below_the_identity() {
    let value = build(INCREMENT, Some(r#""c1""#), &["true"]);
    let length = u64::try_from(bytes(&value).len()).unwrap();
    let error = value.canonical_bytes(Limits::new(length - 1)).unwrap_err();
    assert!(matches!(
        error,
        IdentityRefusal::InputBytes { bound, required } if bound == length - 1 && required > bound
    ));
    assert_eq!(
        value.canonical_bytes(Limits::new(length)).unwrap().len() as u64,
        length
    );
}

/// Trace: FR-120-AC-15
#[trace("TC-920", "FR-120-AC-15")]
#[test]
fn cmp_allocates_nothing_and_sorts_like_the_bytes() {
    let mut values = fixtures();
    for n in 0..8 {
        let digit = n.to_string();
        values.push(build(
            INCREMENT,
            Some(&format!("\"r{n}\"")),
            &[digit.as_str(); 3][..n % 4],
        ));
        values.push(build(&node(&format!("n\\\\{n}")), None, &[]));
    }
    for name in [r"a\nb", r"a\u001fb", "é", r"a\tb", r"a\\b", r#"a\"b"#] {
        let operation = format!(r#"{{"node":"{name}","package":"p"}}"#);
        values.push(ModelTransition::new(operation, None, vec![], BIG).unwrap());
    }
    assert!(values.len() >= 20);

    for left in &values {
        for right in &values {
            let info = allocation_counter::measure(|| {
                std::hint::black_box(left.cmp(right));
                std::hint::black_box(left.partial_cmp(right));
            });
            assert_eq!(info.count_total, 0);
        }
    }
    let control = allocation_counter::measure(|| {
        std::hint::black_box(values[0].canonical_bytes(BIG).unwrap());
    });
    assert!(control.count_total > 0);

    let mut sorted = values.clone();
    sorted.sort();
    let mut sorted_bytes: Vec<Vec<u8>> = values.iter().map(bytes).collect();
    sorted_bytes.sort();
    assert_eq!(sorted.iter().map(bytes).collect::<Vec<_>>(), sorted_bytes);

    for a in &values {
        for b in &values {
            assert_eq!(a.cmp(b), b.cmp(a).reverse());
            assert_eq!(a.cmp(b), bytes(a).cmp(&bytes(b)));
            assert_eq!(a.partial_cmp(b), Some(a.cmp(b)));
            for c in &values {
                if a <= b && b <= c {
                    assert!(a <= c);
                }
            }
        }
    }
}

/// Trace: FR-120-AC-16
#[trace("TC-920", "FR-120-AC-16")]
#[test]
fn deep_nesting_does_not_overflow_the_stack() {
    const DEPTH: usize = 100_000;
    let deep = format!("{}{}", "[".repeat(DEPTH), "]".repeat(DEPTH));
    let handle = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let value = ModelTransition::new(
                text(INCREMENT),
                Some(deep.clone()),
                vec![text("true"), deep],
                BIG,
            )
            .unwrap();
            let direct = to_vec(&value, BIG).unwrap();
            let named = value.canonical_bytes(BIG).unwrap();
            assert_eq!(direct, named);
            assert!(named.len() > 4 * DEPTH);
            let cloned = value.clone();
            assert_eq!(value.cmp(&cloned), Ordering::Equal);
            assert_eq!(value, cloned);
            assert!(!format!("{value:?}").is_empty());
            drop(cloned);
            drop(value);
        })
        .unwrap();
    handle.join().unwrap();
}
