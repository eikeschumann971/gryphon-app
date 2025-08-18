// GUI Service - simplified implementation
use crate::common::ApplicationResult;
use crate::domains::gui::GUICommandActor;

pub struct GUIService {
    command_actor: GUICommandActor,
}

impl GUIService {
    pub fn new(command_actor: GUICommandActor) -> Self {
        Self { command_actor }
    }

    pub async fn create_application(&self, app_id: String, name: String) -> ApplicationResult<()> {
        self.command_actor
            .create_application(app_id, name)
            .await
            .map_err(|e| crate::common::ApplicationError::EventStore(e))?;
        Ok(())
    }
}
