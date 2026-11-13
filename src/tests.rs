use super::*;

fn date(s: &str) -> Date {
    Date::parse(s).unwrap()
}

#[test]
fn calendar() {
    for s in [
        "29.02.1900",
        "31.04.2026",
        "1.01.2026",
        "00.01.2026",
        "01.01.0000",
        "é.01.2026",
    ] {
        assert!(Date::parse(s).is_err());
    }
    assert!(Date::parse("29.02.2000").is_ok());
}

#[test]
fn array_growth_and_shift() {
    let mut list = ArrayList::new();
    assert_eq!(list.items.len(), 8);
    assert_eq!(list.get(0), None);
    for i in 0usize..100 {
        list.push(i.to_string());
        let expected_capacity = (i + 1).next_power_of_two().max(8);
        assert_eq!(list.items.len(), expected_capacity);
        for j in 0..=i {
            assert_eq!(list.get(j), Some(j.to_string().as_str()));
        }
    }
    assert!(list.remove(7));
    assert_eq!(list.get(7), Some("8"));
    assert!(list.remove(0));
    assert_eq!(list.get(97), Some("99"));
    assert!(!list.remove(98));
    assert!(list.remove(97));
    assert_eq!(list.get(97), None);
    while list.len > 0 {
        assert!(list.remove(0));
    }
    assert_eq!(list.get(0), None);
    assert_eq!(list.items.len(), 128);
    assert!(!list.remove(0));
    list.push("reused".into());
    assert_eq!(list.get(0), Some("reused"));
}

#[test]
fn insertion_deletion_and_persistence() {
    let mut d = Diary::new();
    d.add(date("01.01.2026"), "první\nřádek <>&".into())
        .unwrap();
    d.add(date("03.01.2026"), "třetí".into()).unwrap();
    d.current = 0;
    assert!(d.add(date("03.01.2026"), "ne".into()).is_err());
    assert!(d.add(date("31.12.2025"), "ne".into()).is_err());
    d.add(date("02.01.2026"), "druhý".into()).unwrap();
    // A fixed byte buffer lets us test saving without creating a file.
    let mut bytes = [0; 1024];
    let mut output = io::Cursor::new(&mut bytes[..]);
    d.save(&mut output).unwrap();
    let size = output.position() as usize;
    let mut copy = Diary::load(&bytes[..size]).unwrap();
    assert_eq!(copy.day().unwrap().entries.get(0), Some("první\nřádek <>&"));
    assert!(copy.remove(0));
    assert_eq!(copy.day().unwrap().date, date("02.01.2026"));
    copy.current = 1;
    copy.remove(0);
    assert_eq!(copy.current, 0);
    copy.remove(0);
    assert_eq!(copy.len, 0);
    assert!(Diary::load(b"DENIK1\n01.01.2026 1\n999\nx\n".as_slice()).is_err());
}
