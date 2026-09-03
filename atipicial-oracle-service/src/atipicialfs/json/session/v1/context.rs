use super::super::super::super::proto::atipicialfs_v2;
use super::super::super::helpers::push_json_field;
use super::super::super::object::{atipicialfs_json_container_id, atipicialfs_json_object_id};
use super::verbs::{atipicialfs_container_session_verb_name, atipicialfs_object_session_verb_name};

pub(super) fn atipicialfs_json_object_session_context(
    context: &atipicialfs_v2::session::ObjectSessionContext,
) -> Option<String> {
    let mut out = String::from("{ ");
    let mut first = true;

    if context.verb != 0 {
        let verb = atipicialfs_object_session_verb_name(context.verb);
        push_json_field(&mut out, &mut first, "verb", &verb);
    }
    if let Some(target) = context.target.as_ref() {
        // C# SessionToken uses Address (single object), so serialize target as address.
        let json = atipicialfs_json_object_session_target(target).unwrap_or_else(|| "{ }".to_string());
        push_json_field(&mut out, &mut first, "address", &json);
    }

    if first {
        out.push('}');
    } else {
        out.push_str(" }");
    }
    Some(out)
}

fn atipicialfs_json_object_session_target(
    target: &atipicialfs_v2::session::object_session_context::Target,
) -> Option<String> {
    let mut out = String::from("{ ");
    let mut first = true;

    if let Some(container) = target.container.as_ref().and_then(atipicialfs_json_container_id) {
        push_json_field(&mut out, &mut first, "containerID", &container);
    }
    // C# session tokens use a single Address; pick the first object for parity.
    if let Some(object) = target.objects.first().and_then(atipicialfs_json_object_id) {
        push_json_field(&mut out, &mut first, "objectID", &object);
    }

    if first {
        out.push('}');
    } else {
        out.push_str(" }");
    }
    Some(out)
}

// Rationale: AtipicialFs v1 context JSON keeps array projection available for
// protobuf parity even when current request shapes do not include it.
#[allow(dead_code)]
fn atipicialfs_json_object_id_array(ids: &[atipicialfs_v2::refs::ObjectId]) -> Option<String> {
    if ids.is_empty() {
        return None;
    }
    let mut out = String::from("[ ");
    for (idx, id) in ids.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        if let Some(id_json) = atipicialfs_json_object_id(id) {
            out.push_str(&id_json);
        } else {
            out.push_str("{ }");
        }
    }
    out.push_str(" ]");
    Some(out)
}

pub(super) fn atipicialfs_json_container_session_context(
    context: &atipicialfs_v2::session::ContainerSessionContext,
) -> Option<String> {
    let mut out = String::from("{ ");
    let mut first = true;

    if context.verb != 0 {
        let verb = atipicialfs_container_session_verb_name(context.verb);
        push_json_field(&mut out, &mut first, "verb", &verb);
    }
    if context.wildcard {
        push_json_field(&mut out, &mut first, "wildcard", "true");
    }
    if let Some(container) = context
        .container_id
        .as_ref()
        .and_then(atipicialfs_json_container_id)
    {
        push_json_field(&mut out, &mut first, "containerID", &container);
    }

    if first {
        out.push('}');
    } else {
        out.push_str(" }");
    }
    Some(out)
}
