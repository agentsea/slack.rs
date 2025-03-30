use reqwest::Client;
use serde::Deserialize;

/// Simple Slack client
///
/// - `token`: your Slack bot token (e.g. "xoxb-xxxx-....")
/// - `http_client`: a Reqwest Client for making API calls
pub struct SlackClient {
    token: String,
    http_client: Client,
}

impl SlackClient {
    /// Create a new SlackClient with the given bot token
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            http_client: Client::new(),
        }
    }

    // ----------------------------------------------------------------
    //  1) Send a normal channel message (with optional Block Kit)
    // ----------------------------------------------------------------
    /// Posts a message (with optional blocks) to a channel.
    ///
    /// - `channel`: channel ID or name (e.g. "#general" or "C12345")
    /// - `text`: fallback text for notifications, loggers, etc.
    /// - `blocks`: optional JSON array of Block Kit blocks
    ///
    /// Returns the Slack API response or an error.
    pub async fn post_message(
        &self,
        channel: &str,
        text: &str,
        blocks: Option<serde_json::Value>,
    ) -> Result<SlackPostMessageResponse, reqwest::Error> {
        let url = "https://slack.com/api/chat.postMessage";

        // Construct the payload
        let mut body = serde_json::json!({
            "channel": channel,
            "text": text,
        });

        // If we have a JSON array of blocks, insert into the body
        if let Some(blocks_json) = blocks {
            body["blocks"] = blocks_json;
        }

        let resp = self
            .http_client
            .post(url)
            .bearer_auth(&self.token) // pass token as Bearer
            .json(&body)
            .send()
            .await?;

        // Deserialize Slack's JSON response
        let slack_resp = resp.json::<SlackPostMessageResponse>().await?;
        Ok(slack_resp)
    }

    // ----------------------------------------------------------------
    //  2) Send an ephemeral message (visible only to one user)
    // ----------------------------------------------------------------
    /// Posts an ephemeral message in a channel, visible only to `user_id`.
    ///
    /// - `channel`: The channel to post in
    /// - `user_id`: The user who will see the ephemeral message
    /// - `text`: fallback text
    /// - `blocks`: optional JSON array of Block Kit blocks
    ///
    /// Returns the Slack API response or an error.
    pub async fn post_ephemeral(
        &self,
        channel: &str,
        user_id: &str,
        text: &str,
        blocks: Option<serde_json::Value>,
    ) -> Result<SlackEphemeralResponse, reqwest::Error> {
        let url = "https://slack.com/api/chat.postEphemeral";

        let mut body = serde_json::json!({
            "channel": channel,
            "user": user_id,
            "text": text,
        });

        if let Some(blocks_json) = blocks {
            body["blocks"] = blocks_json;
        }

        let resp = self
            .http_client
            .post(url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;

        let slack_resp = resp.json::<SlackEphemeralResponse>().await?;
        Ok(slack_resp)
    }

    // ----------------------------------------------------------------
    //  3) Open a modal (view) in Slack
    // ----------------------------------------------------------------
    /// Opens a modal (View) in Slack.
    ///
    /// NOTE: The `trigger_id` comes from interactive payloads or slash commands.
    ///
    /// - `trigger_id`: Provided by Slack when a user invokes an action (e.g. button click)
    /// - `view`: a JSON object describing the modal’s structure
    ///
    /// Returns the Slack API response or an error.
    pub async fn open_modal(
        &self,
        trigger_id: &str,
        view: serde_json::Value,
    ) -> Result<SlackViewOpenResponse, reqwest::Error> {
        let url = "https://slack.com/api/views.open";

        let body = serde_json::json!({
            "trigger_id": trigger_id,
            "view": view
        });

        let resp = self
            .http_client
            .post(url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;

        let slack_resp = resp.json::<SlackViewOpenResponse>().await?;
        Ok(slack_resp)
    }
}

// --------------------------------------------------------------------
//  Response Structs
// --------------------------------------------------------------------

/// Slack's top-level response object for `chat.postMessage`
#[derive(Debug, Deserialize)]
pub struct SlackPostMessageResponse {
    pub ok: bool,
    pub channel: Option<String>,
    pub ts: Option<String>,
    pub error: Option<String>,
    // If needed, you can add more fields here
}

/// Slack's top-level response object for `chat.postEphemeral`
#[derive(Debug, Deserialize)]
pub struct SlackEphemeralResponse {
    pub ok: bool,
    pub message_ts: Option<String>,
    pub error: Option<String>,
    // ...
}

/// Slack's top-level response for `views.open`
#[derive(Debug, Deserialize)]
pub struct SlackViewOpenResponse {
    pub ok: bool,
    pub view: Option<SlackView>,
    pub error: Option<String>,
}

/// Minimal struct to reflect a Slack View object
#[derive(Debug, Deserialize)]
pub struct SlackView {
    pub id: String,
    // Add other fields as needed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// A helper to see if we have a Slack token available.
    fn maybe_get_slack_token() -> Option<String> {
        env::var("SLACK_BOT_TOKEN").ok()
    }

    #[tokio::test]
    async fn test_post_message() {
        // This test attempts a real Slack API call if SLACK_BOT_TOKEN is set.
        match maybe_get_slack_token() {
            Some(token) => {
                let slack = SlackClient::new(token);
                // Use a channel you can safely post test messages into, e.g., "#random" or a private channel
                let channel = "#orign-bot";
                let text = "dlrow olleh";

                let result = slack.post_message(channel, text, None).await;
                assert!(
                    result.is_ok(),
                    "post_message returned an error: {:?}",
                    result
                );

                let resp = result.unwrap();
                println!("resp: {:?}", resp);
                assert!(resp.ok, "Slack returned an error: {:?}", resp.error);
                assert!(resp.ts.is_some());
            }
            None => {
                println!("Skipping test_post_message because SLACK_BOT_TOKEN is not set.");
            }
        }
    }

    #[tokio::test]
    async fn test_post_interactive_message() {
        // This test attempts a real Slack API call if SLACK_BOT_TOKEN is set.
        match maybe_get_slack_token() {
            Some(token) => {
                let slack = SlackClient::new(token);

                // Use a channel you can safely post test messages into
                let channel = "#orign-bot";
                let text = "Do you want to continue?";

                // Inline JSON blocks with two buttons: "Yes" and "No"
                let blocks = serde_json::json!([
                    {
                        "type": "section",
                        "text": {
                            "type": "mrkdwn",
                            "text": text
                        }
                    },
                    {
                        "type": "actions",
                        "elements": [
                            {
                                "type": "button",
                                "text": {
                                    "type": "plain_text",
                                    "text": "Yes"
                                },
                                "value": "yes",
                                "action_id": "test_button_yes"
                            },
                            {
                                "type": "button",
                                "text": {
                                    "type": "plain_text",
                                    "text": "No"
                                },
                                "value": "no",
                                "action_id": "test_button_no"
                            }
                        ]
                    }
                ]);

                // Use the existing post_message method, passing in the blocks
                let result = slack.post_message(channel, text, Some(blocks)).await;

                // Ensure the request didn’t fail
                assert!(
                    result.is_ok(),
                    "Sending interactive message returned an error: {:?}",
                    result
                );

                // Confirm Slack's JSON response
                let resp = result.unwrap();
                println!("Interactive message response: {:?}", resp);

                // Confirm Slack's "ok" field
                assert!(resp.ok, "Slack returned an error: {:?}", resp.error);

                // Confirm we got a valid timestamp
                assert!(
                    resp.ts.is_some(),
                    "No timestamp was returned for interactive message."
                );
            }
            None => {
                println!("Skipping test_post_interactive_message_inline because SLACK_BOT_TOKEN is not set.");
            }
        }
    }

    // #[tokio::test]
    // async fn test_post_ephemeral() {
    //     // This test also attempts a real Slack API call if SLACK_BOT_TOKEN is set.
    //     match maybe_get_slack_token() {
    //         Some(token) => {
    //             let slack = SlackClient::new(token);
    //             // Replace with your known channel and test user ID
    //             let channel = "#general";
    //             let user_id = "U123456";
    //             let text = "Hello ephemeral test!";

    //             let result = slack.post_ephemeral(channel, user_id, text, None).await;
    //             assert!(
    //                 result.is_ok(),
    //                 "post_ephemeral returned an error: {:?}",
    //                 result
    //             );

    //             let resp = result.unwrap();
    //             assert!(resp.ok, "Slack returned an error: {:?}", resp.error);
    //             // ephemeral responses won't necessarily have a channel in the same format,
    //             // so we'll just check if we at least didn't get an error.
    //         }
    //         None => {
    //             println!("Skipping test_post_ephemeral because SLACK_BOT_TOKEN is not set.");
    //         }
    //     }
    // }

    // #[tokio::test]
    // async fn test_open_modal() {
    //     // This test attempts to open a modal if SLACK_BOT_TOKEN is set.
    //     match maybe_get_slack_token() {
    //         Some(token) => {
    //             let slack = SlackClient::new(token);
    //             // Real "trigger_id" is needed from an interactive Slack action
    //             // For demonstration purposes, it will likely fail unless you
    //             // provide a real short-lived trigger_id from Slack
    //             let fake_trigger_id = "12345.98765.abcd2358f";

    //             let my_modal_view = serde_json::json!({
    //                 "type": "modal",
    //                 "title": {
    //                     "type": "plain_text",
    //                     "text": "Example Modal"
    //                 },
    //                 "close": {
    //                     "type": "plain_text",
    //                     "text": "Close"
    //                 },
    //                 "blocks": [
    //                     {
    //                         "type": "section",
    //                         "text": {
    //                             "type": "mrkdwn",
    //                             "text": "This is a test modal."
    //                         }
    //                     }
    //                 ]
    //             });

    //             let result = slack.open_modal(fake_trigger_id, my_modal_view).await;
    //             // In a real scenario, you'd have a genuine trigger_id from Slack. This will likely fail without it.
    //             // For demonstration:
    //             println!("Modal call result: {:?}", result);
    //         }
    //         None => {
    //             println!("Skipping test_open_modal because SLACK_BOT_TOKEN is not set.");
    //         }
    //     }
    // }
}
