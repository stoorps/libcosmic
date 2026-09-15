use super::*;
use iced_accessibility::accesskit::{Action, ActionRequest, NodeId, Role, TreeId};

fn dropdown(options: &'static [&'static str]) -> Dropdown<'static, &'static str, usize, usize> {
    Dropdown::new(Cow::Borrowed(options), Some(0), |i| i)
        .name("Filesystem")
        .id(Id::new("filesystem"))
}
fn click(id: &Id) -> Event {
    let n = u64::from(id.clone());
    Event::A11y(
        Id::from(n),
        ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: NodeId(n),
            data: None,
        },
    )
}
fn key(named: keyboard::key::Named) -> Event {
    let key = keyboard::Key::Named(named);
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Tab),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: None,
        repeat: false,
    })
}
#[test]
fn actual_control_node_has_stable_name_identity_value_and_disabled_semantics() {
    for options in [&["ext4", "btrfs"][..], &[][..]] {
        let control = dropdown(options);
        let tree = Tree::new(&control as &dyn Widget<usize, crate::Theme, crate::Renderer>);
        let layout = layout::Node::new(Size::new(150.0, 30.0));
        let nodes = control.a11y_nodes(Layout::new(&layout), &tree, mouse::Cursor::Unavailable);
        let node = nodes.root()[0].node();
        assert_eq!(node.role(), Role::ComboBox);
        assert_eq!(node.label(), Some("Filesystem"));
        assert_eq!(node.author_id(), Some("filesystem"));
        assert_eq!(node.value(), Some(options.first().copied().unwrap_or("")));
        assert_eq!(node.is_disabled(), options.is_empty());
        assert_eq!(node.supports_action(Action::Click), !options.is_empty());
        assert_eq!(node.is_expanded(), Some(false));
        assert!(nodes.children().is_empty());
    }
}
#[test]
fn targeted_control_actions_request_existing_open_close_lifecycle() {
    let control = dropdown(&["ext4"]);
    let mut state = State::new();
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    control.control_event(&click(&Id::unique()), &mut state, &mut shell);
    assert!(!state.open_operation);
    assert!(!shell.is_event_captured());
    state.close_operation = true;
    control.control_event(&click(control.id.as_ref().unwrap()), &mut state, &mut shell);
    assert!(state.open_operation);
    assert!(!state.close_operation);
    assert!(shell.is_event_captured());
    state.open_operation = false;
    state.is_open.store(true, Ordering::Relaxed);
    control.control_event(&click(control.id.as_ref().unwrap()), &mut state, &mut shell);
    assert!(state.close_operation);
    assert!(!state.open_operation);
    assert!(messages.is_empty());
    let empty = dropdown(&[]);
    state.open_operation = false;
    empty.control_event(
        &click(empty.id.as_ref().unwrap()),
        &mut state,
        &mut Shell::new(&mut messages),
    );
    assert!(!state.open_operation);
}
#[test]
fn keyboard_selection_requires_focus_and_respects_boundaries() {
    let control = dropdown(&["ext4", "btrfs"]);
    let mut state = State::new();
    let mut messages = Vec::new();
    let down = key(keyboard::key::Named::ArrowDown);
    control.control_event(&down, &mut state, &mut Shell::new(&mut messages));
    assert!(messages.is_empty());
    iced_core::widget::operation::Focusable::focus(&mut state);
    control.control_event(&down, &mut state, &mut Shell::new(&mut messages));
    assert_eq!(messages, [1]);
    control.control_event(
        &key(keyboard::key::Named::ArrowUp),
        &mut state,
        &mut Shell::new(&mut messages),
    );
    assert_eq!(messages, [1]);
    control.control_event(
        &key(keyboard::key::Named::Enter),
        &mut state,
        &mut Shell::new(&mut messages),
    );
    assert!(state.open_operation);
    state.is_open.store(true, Ordering::Relaxed);
    control.control_event(
        &key(keyboard::key::Named::Escape),
        &mut state,
        &mut Shell::new(&mut messages),
    );
    assert!(state.close_operation);
    let mut shell = Shell::new(&mut messages);
    control.control_event(&key(keyboard::key::Named::Tab), &mut state, &mut shell);
    assert!(!shell.is_event_captured());
}
