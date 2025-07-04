#![allow(unused_variables)]

#[cfg(test)]
mod uid_conversion {
    use crate::domain::value_types::uid::Uid;

    /// Value type for Task Unique IDs
    type TaskUid = Uid<Task>;

    /// Task struct using the value type
    struct Task {
        task_uid: TaskUid,
    }

    /// Task DTO object
    /// - This is useful when converting between application internals, and serialising back to primitive values. This is typically either when interfacing with `sqlx` or writing to disk with Serde.
    #[allow(dead_code)]
    struct TaskDto {
        pub uuid: uuid::Uuid,
    }

    impl From<Task> for TaskDto {
        fn from(value: Task) -> TaskDto {
            TaskDto {
                uuid: value.task_uid.into(),
            }
        }
    }

    #[test]
    fn convert_to_raw_uuid() {
        let uid = TaskUid::from(uuid::Uuid::new_v4());
        let task = Task { task_uid: uid };

        // We can use the inverse of `From<T>`, to call `Into::into()`.
        let raw_uid: uuid::Uuid = task.task_uid.into();
    }

    #[test]
    fn convert_and_map_collection_to_dto() {
        let uid = TaskUid::from(uuid::Uuid::new_v4());
        let task1 = Task { task_uid: uid };
        let task2 = Task { task_uid: uid };

        // Assume we have a collection of tasks, which we intend to serialise to disk
        let tasks = vec![task1, task2];

        // Here, we convert and map the objects, creating a new collection
        // Our vector of `Task` items are now a vector of `TaskDto`.
        //
        // `TaskDto` is easily serializable now.
        let mapped_tasks = tasks.into_iter().map(Into::into).collect::<Vec<TaskDto>>();
    }
}
