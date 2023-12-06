use super::*;

#[test]
fn test_join_channels() {
    let mut channels = Channels::new();

    channels.join_user("#room1", "bob");
    channels.join_user("#room2", "bob");
    channels.join_user("#room3", "bob");

    let expected_rooms = vec!["#room1", "#room2", "#room3"]
        .iter()
        .map(|s| s.to_string())
        .collect::<HashSet<_>>();
    let actual_rooms = channels
        .channels_map
        .keys()
        .map(|s| s.to_owned())
        .collect::<HashSet<_>>();

    assert_eq!(expected_rooms, actual_rooms);
}

#[test]
fn test_list_users() {
    let mut channels = Channels::new();

    channels.join_user("#room1", "bob");
    channels.join_user("#room1", "ana");
    channels.join_user("#room1", "ricardo");

    let expected_users = vec!["bob", "ana", "ricardo"]
        .iter()
        .map(|s| s.to_string())
        .collect::<HashSet<_>>();

    let actual_users = channels
        .channel_list("#room1")
        .map(|s| s.to_owned())
        .collect::<HashSet<_>>();
    
    assert_eq!(expected_users, actual_users);

    assert!(channels.channel_list("#room2").count() == 0);
}

