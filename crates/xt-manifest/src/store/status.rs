use crate::model::OperationStatus;

pub(super) fn put_status(statuses: &mut Vec<OperationStatus>, status: OperationStatus) {
    if let Some(previous) = statuses.iter_mut().find(|old| old.same_key(&status)) {
        let newer = previous
            .updated_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());
        let incoming = status
            .updated_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());
        if newer.zip(incoming).is_some_and(|(old, new)| old > new) {
            return;
        }
        *previous = status;
    } else {
        statuses.push(status);
    }
}
