use gitlab_cli::models::user::AccessLevel;
use std::str::FromStr;

#[test]
fn test_access_level_from_str() {
    // Test all valid access level strings
    assert_eq!(AccessLevel::from_str("noaccess").unwrap().as_u64(), 0);
    assert_eq!(AccessLevel::from_str("no_access").unwrap().as_u64(), 0);
    assert_eq!(AccessLevel::from_str("no-access").unwrap().as_u64(), 0);
    assert_eq!(AccessLevel::from_str("0").unwrap().as_u64(), 0);

    assert_eq!(AccessLevel::from_str("minimalaccess").unwrap().as_u64(), 5);
    assert_eq!(AccessLevel::from_str("minimal_access").unwrap().as_u64(), 5);
    assert_eq!(AccessLevel::from_str("minimal-access").unwrap().as_u64(), 5);
    assert_eq!(AccessLevel::from_str("5").unwrap().as_u64(), 5);

    assert_eq!(AccessLevel::from_str("guest").unwrap().as_u64(), 10);
    assert_eq!(AccessLevel::from_str("10").unwrap().as_u64(), 10);

    assert_eq!(AccessLevel::from_str("planner").unwrap().as_u64(), 15);
    assert_eq!(AccessLevel::from_str("15").unwrap().as_u64(), 15);

    assert_eq!(AccessLevel::from_str("reporter").unwrap().as_u64(), 20);
    assert_eq!(AccessLevel::from_str("20").unwrap().as_u64(), 20);

    assert_eq!(AccessLevel::from_str("developer").unwrap().as_u64(), 30);
    assert_eq!(AccessLevel::from_str("30").unwrap().as_u64(), 30);

    assert_eq!(AccessLevel::from_str("maintainer").unwrap().as_u64(), 40);
    assert_eq!(AccessLevel::from_str("40").unwrap().as_u64(), 40);

    assert_eq!(AccessLevel::from_str("owner").unwrap().as_u64(), 50);
    assert_eq!(AccessLevel::from_str("50").unwrap().as_u64(), 50);

    // Test with uppercase and mixed case
    assert_eq!(AccessLevel::from_str("Developer").unwrap().as_u64(), 30);
    assert_eq!(AccessLevel::from_str("MAINTAINER").unwrap().as_u64(), 40);

    // Test invalid access level
    assert!(AccessLevel::from_str("invalid").is_err());
    assert!(AccessLevel::from_str("60").is_err());
}

#[test]
fn test_access_level_as_u64() {
    assert_eq!(AccessLevel::NoAccess.as_u64(), 0);
    assert_eq!(AccessLevel::MinimalAccess.as_u64(), 5);
    assert_eq!(AccessLevel::Guest.as_u64(), 10);
    assert_eq!(AccessLevel::Planner.as_u64(), 15);
    assert_eq!(AccessLevel::Reporter.as_u64(), 20);
    assert_eq!(AccessLevel::Developer.as_u64(), 30);
    assert_eq!(AccessLevel::Maintainer.as_u64(), 40);
    assert_eq!(AccessLevel::Owner.as_u64(), 50);
}

#[test]
fn test_access_level_display() {
    assert_eq!(format!("{}", AccessLevel::NoAccess), "No Access");
    assert_eq!(format!("{}", AccessLevel::MinimalAccess), "Minimal Access");
    assert_eq!(format!("{}", AccessLevel::Guest), "Guest");
    assert_eq!(format!("{}", AccessLevel::Planner), "Planner");
    assert_eq!(format!("{}", AccessLevel::Reporter), "Reporter");
    assert_eq!(format!("{}", AccessLevel::Developer), "Developer");
    assert_eq!(format!("{}", AccessLevel::Maintainer), "Maintainer");
    assert_eq!(format!("{}", AccessLevel::Owner), "Owner");
}
