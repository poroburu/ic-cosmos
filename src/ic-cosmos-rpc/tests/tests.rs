mod setup;

use std::collections::HashMap;

use ic_cosmos::{
    metrics::{MetricRpcHost, Metrics},
    request::RpcRequest,
    rpc_client::RpcServices,
    types::Cluster,
};
use ic_cosmos_rpc::{auth::Auth, state::InitArgs, types::RegisterProviderArgs};
use test_utils::{MockOutcallBuilder, TestSetup};

use crate::setup::{mock_update, CosmosRpcSetup, MOCK_RAW_TX};

#[test]
fn should_canonicalize_json_response() {
    let setup = CosmosRpcSetup::default();
    let responses = [
        r#"{"id":1,"jsonrpc":"2.0","result":"ok"}"#,
        r#"{"result":"ok","id":1,"jsonrpc":"2.0"}"#,
        r#"{"result":"ok","jsonrpc":"2.0","id":1}"#,
    ]
    .iter()
    .map(|&response| {
        setup
            .request(RpcServices::Mainnet, "getHealth", "", 1000)
            .mock_http(MockOutcallBuilder::new(200, response))
            .wait()
    })
    .collect::<Vec<_>>();
    assert!(responses.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn test_get_health() {
    assert_eq!(
        mock_update::<_, String>(
            "cos_getHealth",
            (RpcServices::Mainnet, ()),
            r#"{"jsonrpc":"2.0","result":"ok","id":1}"#,
        )
        .unwrap(),
        "ok"
    );

    assert!(mock_update::<_, String>(
        "cos_getHealth",
        (RpcServices::Mainnet, ()),
        r#"{"jsonrpc":"2.0","error":{"code":-32005,"message":"Node is unhealthy","data":{}},"id":1}"#,
    )
    .is_err());
}

#[test]
fn should_get_valid_request_cost() {
    assert_eq!(
        CosmosRpcSetup::new(InitArgs {
            demo: None,
            ..Default::default()
        })
        .call_query::<_, u128>("requestCost", (MOCK_RAW_TX, 1000u64)),
        321476800
    );
}

#[test]
fn should_get_nodes_in_subnet() {
    assert_eq!(CosmosRpcSetup::default().get_nodes_in_subnet(), 34);
}

#[test]
fn should_allow_manager_to_authorize_and_deauthorize_user() {
    let setup = CosmosRpcSetup::default();
    let principal = TestSetup::principal(3);

    setup
        .clone()
        .as_controller()
        .authorize(principal, Auth::RegisterProvider)
        .wait();
    let principals = setup.get_authorized(Auth::RegisterProvider);
    assert!(principals.contains(&principal));
    setup
        .clone()
        .as_controller()
        .deauthorize(principal, Auth::RegisterProvider)
        .wait();
    let principals = setup.get_authorized(Auth::RegisterProvider);
    assert!(!principals.contains(&principal));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn should_not_allow_caller_without_access_authorize_users() {
    CosmosRpcSetup::default()
        .authorize(TestSetup::principal(9), Auth::RegisterProvider)
        .wait();
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn should_not_allow_caller_without_access_deauthorize_users() {
    CosmosRpcSetup::default()
        .deauthorize(TestSetup::principal(9), Auth::RegisterProvider)
        .wait();
}

#[test]
fn should_allow_manager_to_register_and_unregister_providers() {
    let setup = CosmosRpcSetup::default();
    let provider_id = "test_mainnet1".to_string();
    setup
        .clone()
        .as_controller()
        .register_provider(RegisterProviderArgs {
            id: provider_id.clone(),
            url: Cluster::Mainnet.url().into(),
            auth: None,
        })
        .wait();
    let providers = setup.get_providers();
    assert!(providers.contains(&provider_id));
    setup.clone().as_controller().unregister_provider(&provider_id).wait();
    let providers = setup.get_providers();
    assert!(!providers.contains(&provider_id));
}

#[test]
fn should_allow_caller_with_access_register_provider() {
    let setup = CosmosRpcSetup::default();
    let principal = TestSetup::principal(3);

    setup
        .clone()
        .as_controller()
        .authorize(principal, Auth::RegisterProvider)
        .wait();

    let provider_id = "test_mainnet1".to_string();
    setup
        .clone()
        .as_caller(principal)
        .register_provider(RegisterProviderArgs {
            id: provider_id.clone(),
            url: Cluster::Mainnet.url().into(),
            auth: None,
        })
        .wait();
    let providers = setup.get_providers();
    assert!(providers.contains(&provider_id));
    setup
        .clone()
        .as_caller(principal)
        .unregister_provider(&provider_id)
        .wait();
    let providers = setup.get_providers();
    assert!(!providers.contains(&provider_id));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn should_not_allow_caller_without_access_to_register_provider() {
    CosmosRpcSetup::default()
        .register_provider(RegisterProviderArgs {
            id: "test_mainnet1".to_string(),
            url: Cluster::Mainnet.url().into(),
            auth: None,
        })
        .wait();
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn should_not_allow_caller_without_access_to_unregister_provider() {
    CosmosRpcSetup::default().unregister_provider("mainnet").wait();
}

#[test]
fn should_retrieve_logs() {
    let setup = CosmosRpcSetup::new(InitArgs {
        demo: None,
        ..Default::default()
    });
    assert_eq!(setup.http_get_logs("DEBUG"), vec![]);
    assert_eq!(setup.http_get_logs("INFO"), vec![]);

    let principal = TestSetup::principal(3);

    setup
        .clone()
        .as_controller()
        .authorize(principal, Auth::RegisterProvider)
        .wait();

    assert_eq!(setup.http_get_logs("DEBUG"), vec![]);
    assert!(setup.http_get_logs("INFO")[0].message.contains(
        format!(
            "Authorizing `{:?}` for principal: {}",
            Auth::RegisterProvider,
            principal
        )
        .as_str()
    ));
}

#[test]
fn should_recognize_rate_limit() {
    let setup = CosmosRpcSetup::default();
    let result = setup
        .request(RpcServices::Mainnet, "getHealth", "", 1000)
        .mock_http(MockOutcallBuilder::new(
            429,
            r#"{"jsonrpc":"2.0","error":{"code":429,"message":"Too many requests for a specific RPC call"},"id":1}"#,
        ))
        .wait();

    println!("{:#?}", result);

    // TODO: fix
    // assert_eq!(
    //     result,
    //     Err(RpcError::HttpOutcallError {
    //         code: 429.into(),
    //         message: "(Rate limit error message)".to_string(),
    //     })
    // );

    let rpc_method = || RpcRequest::GetHealth.into();
    let host = MetricRpcHost(Cluster::Mainnet.host_str().unwrap());

    assert_eq!(
        setup.get_metrics(),
        Metrics {
            requests: [((rpc_method(), host.clone()), 1)]
                .into_iter()
                .collect::<HashMap<_, _>>(),
            responses: [((rpc_method(), host, 429.into()), 1)]
                .into_iter()
                .collect::<HashMap<_, _>>(),
            ..Default::default()
        }
    );
}

#[test]
fn upgrade_should_keep_state() {
    let setup = CosmosRpcSetup::default();
    let principal = TestSetup::principal(3);

    setup
        .clone()
        .as_controller()
        .authorize(principal, Auth::RegisterProvider)
        .wait();

    let principals = setup.get_authorized(Auth::RegisterProvider);
    assert!(principals.contains(&principal));

    setup
        .clone()
        .as_controller()
        .register_provider(RegisterProviderArgs {
            id: "test_mainnet1".to_string(),
            url: Cluster::Mainnet.url().into(),
            auth: None,
        })
        .wait();

    let providers = setup.get_providers();
    assert!(providers.contains(&"test_mainnet1".to_string()));

    setup.upgrade_canister(InitArgs::default());

    let principals = setup.get_authorized(Auth::RegisterProvider);
    assert!(principals.contains(&principal));

    let providers = setup.get_providers();
    assert!(providers.contains(&"test_mainnet1".to_string()));
}
