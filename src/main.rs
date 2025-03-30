use slack_http_client::SlackClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Provide your bot token (export SLACK_BOT_TOKEN=...)
    let slack_token =
        std::env::var("SLACK_BOT_TOKEN").expect("Missing SLACK_BOT_TOKEN environment variable");

    // 2) Instantiate the SlackClient
    let slack_client = SlackClient::new(slack_token);

    // 3) Prepare a Block Kit array with a button
    let blocks_with_button = serde_json::json!([
        {
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": "Hello, world! Here’s a button."
            },
            "accessory": {
                "type": "button",
                "text": {
                    "type": "plain_text",
                    "text": "Click me"
                },
                "value": "my_button_value",
                "action_id": "button_click"
            }
        }
    ]);

    // 4) Post a channel message with that block
    let channel_id = "#general"; // or "C1234567" if you know the ID
    let post_resp = slack_client
        .post_message(
            channel_id,
            "Hello, world fallback",
            Some(blocks_with_button),
        )
        .await?;

    if post_resp.ok {
        println!(
            "Successfully posted message to channel: {:?}",
            post_resp.channel
        );
    } else {
        eprintln!("Failed to post message: {:?}", post_resp.error);
    }

    // 5) (Optional) Post an ephemeral message to a user in that channel
    //    Suppose you know the user’s Slack user ID: "U123456"
    let ephemeral_resp = slack_client
        .post_ephemeral(channel_id, "U123456", "Hello ephemeral", None)
        .await?;

    if ephemeral_resp.ok {
        println!("Ephemeral posted: {:?}", ephemeral_resp.message_ts);
    } else {
        eprintln!("Failed to post ephemeral: {:?}", ephemeral_resp.error);
    }

    // 6) (Optional) Open a modal view
    //
    //    *In reality, you'd only have a valid `trigger_id`
    //    from an interactive payload or slash command.
    //    For example’s sake, we mock a trigger_id:
    let fake_trigger_id = "12345.98765.abcd2358f"; // Slack usually sends a short-lived one

    // Construct the modal view:
    let my_modal_view = serde_json::json!({
        "type": "modal",
        "title": {
            "type": "plain_text",
            "text": "Example Modal"
        },
        "close": {
            "type": "plain_text",
            "text": "Close"
        },
        "blocks": [
            {
                "type": "input",
                "block_id": "block_input",
                "element": {
                    "type": "plain_text_input",
                    "action_id": "my_input_action"
                },
                "label": {
                    "type": "plain_text",
                    "text": "Say something"
                }
            }
        ]
    });

    let modal_resp = slack_client
        .open_modal(fake_trigger_id, my_modal_view)
        .await?;

    if modal_resp.ok {
        println!("Modal opened: {:?}", modal_resp.view);
    } else {
        eprintln!("Failed to open modal: {:?}", modal_resp.error);
    }

    Ok(())
}
