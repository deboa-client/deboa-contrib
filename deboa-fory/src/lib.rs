#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
use deboa::{
    Result,
    errors::{ContentError, DeboaError},
    request::DeboaRequestBuilder,
    response::DeboaResponse,
};
use fory::{Fory, Serializer};
use http::header;

//#[cfg(test)]
//mod tests;

/// Fory request builder extension
pub trait ForyRequestBuilder {
    /// Set the request body as Fory
    fn body_as_fory<T: Serializer<Target = T>>(
        self,
        fory: &Fory,
        body: T,
    ) -> Result<DeboaRequestBuilder>;
}

impl ForyRequestBuilder for DeboaRequestBuilder {
    fn body_as_fory<T: Serializer<Target = T>>(
        self,
        fory: &Fory,
        body: T,
    ) -> Result<DeboaRequestBuilder> {
        let result = fory.serialize(&body);
        let Ok(data) = result else {
            return Err(DeboaError::Content(ContentError::Serialization {
                message: result
                    .unwrap_err()
                    .to_string(),
            }));
        };

        println!("data: {:?}", data);

        self.bytes(&data)
            .header(header::CONTENT_TYPE, "application/fory")
    }
}

/// Fory response extension
pub trait ForyResponse {
    /// Get the response body as Fory
    fn body_as_fory<T: Serializer<Target = T> + std::fmt::Debug>(
        self,
        fory: &Fory,
    ) -> impl std::future::Future<Output = Result<T>>;
}

impl ForyResponse for DeboaResponse {
    async fn body_as_fory<T: Serializer<Target = T> + std::fmt::Debug>(
        self,
        fory: &Fory,
    ) -> Result<T> {
        let result = fory.deserialize(&self.bytes().await?);
        let Ok(data) = result else {
            return Err(DeboaError::Content(ContentError::Deserialization {
                message: result
                    .unwrap_err()
                    .to_string(),
            }));
        };

        Ok(data)
    }
}
