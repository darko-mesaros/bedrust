pub mod captioner;
pub mod chat;
pub mod code;
pub mod config;
pub mod constants;
pub mod models;
pub mod utils;

use aws_config::environment::credentials::EnvironmentVariableCredentialsProvider;
use aws_config::imds::credentials::ImdsCredentialsProvider;
use aws_config::meta::credentials::CredentialsProviderChain;
use aws_config::meta::region::RegionProviderChain;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_config::profile::ProfileFileRegionProvider;
use aws_config::BehaviorVersion;
use aws_types::region::Region;

//======================================== AWS_REGION
// FIX: Return Result
pub async fn configure_aws(
    fallback_region: String,
    profile_name: &String,
) -> aws_config::SdkConfig {
    let region_provider = RegionProviderChain::first_try(
        ProfileFileRegionProvider::builder()
            .profile_name(profile_name)
            .build(),
    )
    .or_else(aws_config::environment::EnvironmentVariableRegionProvider::new())
    .or_else(aws_config::imds::region::ImdsRegionProvider::builder().build())
    .or_else(Region::new(fallback_region));

    let credentials_provider = CredentialsProviderChain::first_try(
        "Environment",
        EnvironmentVariableCredentialsProvider::new(),
    )
    .or_else(
        "Profile",
        ProfileFileCredentialsProvider::builder()
            .profile_name(profile_name)
            .build(),
    )
    .or_else("IMDS", ImdsCredentialsProvider::builder().build());

    aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials_provider)
        .region(region_provider)
        .load()
        .await
}
//======================================== END AWS
