use thiserror::Error;

/// Error types for OATS operations
#[derive(Error, Debug)]
pub enum OatsError {
    #[error("Object not found: {id}")]
    ObjectNotFound { id: String },

    #[error("Trait not found: {trait_name}")]
    TraitNotFound { trait_name: String },

    #[error("Action failed: {message}")]
    ActionFailed { message: String },

    #[error("System error: {message}")]
    SystemError { message: String },

    #[error("Invalid state: {message}")]
    InvalidState { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Resource exhausted: {message}")]
    ResourceExhausted { message: String },

    #[error("Timeout error: {message}")]
    TimeoutError { message: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl OatsError {
    /// Create a new object not found error
    pub fn object_not_found(id: impl Into<String>) -> Self {
        Self::ObjectNotFound { id: id.into() }
    }

    /// Create a new trait not found error
    pub fn trait_not_found(trait_name: impl Into<String>) -> Self {
        Self::TraitNotFound {
            trait_name: trait_name.into(),
        }
    }

    /// Create a new action failed error
    pub fn action_failed(message: impl Into<String>) -> Self {
        Self::ActionFailed {
            message: message.into(),
        }
    }

    /// Create a new system error
    pub fn system_error(message: impl Into<String>) -> Self {
        Self::SystemError {
            message: message.into(),
        }
    }

    /// Create a new invalid state error
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::InvalidState {
            message: message.into(),
        }
    }

    /// Create a new unknown error
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::Unknown {
            message: message.into(),
        }
    }

    /// Create a new validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }

    /// Create a new resource exhausted error
    pub fn resource_exhausted(message: impl Into<String>) -> Self {
        Self::ResourceExhausted {
            message: message.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::TimeoutError {
            message: message.into(),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            OatsError::ObjectNotFound { .. }
                | OatsError::TraitNotFound { .. }
                | OatsError::ValidationError { .. }
                | OatsError::TimeoutError { .. }
        )
    }

    /// Check if this error is fatal
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            OatsError::ResourceExhausted { .. }
                | OatsError::InvalidState { .. }
                | OatsError::SystemError { .. }
        )
    }
}
