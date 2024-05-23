// HTTP API interface
use dropshot::{
    endpoint, HttpError, HttpResponseOk, HttpResponseUpdatedNoContent, Path, RequestContext,
    TypedBody,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::{context::Context, entity::counter::Counter};

/// `CounterValue` represents the value of the API's counter, either as the
/// response to a GET request to fetch the counter or as the body of a PUT
/// request to update the counter.
#[derive(Deserialize, Serialize, JsonSchema)]
struct CounterValue {
    counter: Option<u32>,
}

#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct PathParams {
    key: Uuid,
}

/// Fetch the current value of the counter.
#[endpoint {
    method = GET,
    path = "/counter/{key}",
}]
pub async fn get_counter(
    rqctx: RequestContext<Context>,
    path_params: Path<PathParams>,
) -> Result<HttpResponseOk<CounterValue>, HttpError> {
    let context = rqctx.context();
    let key = path_params.into_inner().key;

    let counter = context
        .database()
        .read(move |connection| {
            let tx = connection.transaction()?;
            Ok(Counter::retrieve(&tx, &key)?)
        })
        .await?;

    Ok(HttpResponseOk(CounterValue {
        counter: counter.map(|c| c.value),
    }))
}

/// Update the current value of the counter.
#[endpoint {
    method = PUT,
    path = "/counter/{key}",
}]
pub async fn put_counter(
    rqctx: RequestContext<Context>,
    path_params: Path<PathParams>,
    update: TypedBody<CounterValue>,
) -> Result<HttpResponseUpdatedNoContent, HttpError> {
    let context = rqctx.context();
    let key = path_params.into_inner().key;
    let updated_value = update.into_inner();

    match updated_value.counter {
        Some(counter) => {
            context
                .database()
                .write(move |connection| {
                    let tx = connection.transaction()?;
                    Counter::new(key, counter).upsert(&tx)?;
                    Ok(tx.commit()?)
                })
                .await?;

            Ok(HttpResponseUpdatedNoContent())
        }
        None => Err(HttpError::for_bad_request(
            None,
            "No value provided".to_string(),
        )),
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::routes::{
        client::{types::CounterValue, Client},
        test::TestContext,
    };
    use anyhow::Result;
    use uuid::Uuid;

    #[tokio::test]
    async fn roundtrip() -> Result<()> {
        let context = TestContext::new(vec![]).await?;
        let client = Client::new(&format!("http://{}", context.bind_address()));

        let key = Uuid::from_str("029d1abe-c121-4b54-806b-a692e101a5ea").unwrap();

        // initial value will not be set
        let counter_value = client.get_counter(&key).await?.into_inner();
        assert!(counter_value.counter.is_none());

        // set value
        client
            .put_counter(&key, &CounterValue { counter: Some(11) })
            .await?;

        // retrieve value
        let counter_value = client.get_counter(&key).await?.into_inner();
        assert_eq!(counter_value.counter, Some(11));

        Ok(())
    }
}
