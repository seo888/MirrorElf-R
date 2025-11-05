// aws-config = { version = "1.6.0", features = ["behavior-version-latest"] }
// aws-sdk-s3 = { version = "1.79.0", features = ["behavior-version-latest"] }
// aws-credential-types = "1.2.2"

use aws_config::SdkConfig;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials; // 更新为正确的 Credentials 导入
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use aws_sdk_s3::{Client, Error};
pub struct MinioClient {
    pub client: Client,
}

impl MinioClient {
    pub fn new(endpoint: &str, access_key: &str, secret_key: &str, secure: bool) -> Self {
        let credentials =
            Credentials::new(access_key, secret_key, None, None, "custom-minio-provider");

        let sdk_config = SdkConfig::builder()
            .credentials_provider(SharedCredentialsProvider::new(credentials))
            .region(aws_config::Region::new("us-east-1"))
            .endpoint_url(if secure {
                format!("https://{}", endpoint)
            } else {
                format!("http://{}", endpoint)
            })
            .build();

        let client = Client::new(&sdk_config);
        MinioClient { client }
    }

    pub async fn delete_all_versions(&self, bucket: &str, prefix: &str) -> Result<(), Error> {
        let mut next_token: Option<String> = None;
        
        loop {
            // println!("bucket:{} prefix:{}", bucket, prefix);
            // 获取对象版本列表
            let list_result = self
                .client
                .list_object_versions()
                .bucket(bucket)
                .prefix(prefix)
                .set_key_marker(next_token)
                .send()
                .await?;

            // println!("list_result:{:?}",list_result);

            // 收集所有版本和删除标记
            let mut objects_to_delete: Vec<ObjectIdentifier> = Vec::new();

            // 添加版本
            if let Some(versions) = list_result.versions {
                for version in versions {
                    objects_to_delete.push(
                        ObjectIdentifier::builder()
                            .key(version.key.unwrap_or_default().to_string())
                            .version_id(version.version_id.unwrap_or_default().to_string())
                            .build()
                            .unwrap(),
                    );
                }
            }

            // 添加删除标记
            if let Some(delete_markers) = list_result.delete_markers {
                for marker in delete_markers {
                    println!("marker: {:?}", marker);
                    objects_to_delete.push(
                        ObjectIdentifier::builder()
                            .key(marker.key.unwrap_or_default().to_string())
                            .version_id(marker.version_id.unwrap_or_default().to_string())
                            .build()
                            .unwrap(),
                    );
                }
            }

            // 如果有对象需要删除
            if !objects_to_delete.is_empty() {
                let delete = Delete::builder()
                    .set_objects(Some(objects_to_delete))
                    .build()
                    .unwrap();

                self.client
                    .delete_objects()
                    .bucket(bucket)
                    .delete(delete)
                    .send()
                    .await?;
            }
            // println!("list_result:{:?}",list_result);
            next_token = list_result.next_key_marker.map(String::from);
            if next_token.is_none() {
                break;
            }
        }

        Ok(())
    }
}

// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     let minio_client = MinioClient::new(
//         "localhost:9000",
//         "your-access-key",
//         "your-secret-key",
//         false,
//     );

//     minio_client
//         .delete_all_versions("my-bucket", Some("prefix/"))
//         .await?;

//     println!("All versions deleted successfully");
//     Ok(())
// }
