use anyhow::Result;
use log::warn;
use teloxide::{
    prelude::*,
    types::{
        CallbackQuery, ChatId, ChatMemberKind, InlineKeyboardButton, InlineKeyboardMarkup,
        Recipient,
    },
};

use crate::{commands::Command, messages};

pub async fn handle_start(bot: Bot, msg: Message, _command: Command) -> Result<()> {
    bot.send_message(msg.chat.id, messages::start_message())
        .reply_markup(ready_keyboard())
        .await?;

    Ok(())
}

pub async fn handle_done(bot: Bot, query: CallbackQuery) -> Result<()> {
    let Some(data) = query.data.as_deref() else {
        return Ok(());
    };

    if data != messages::DONE_CALLBACK_DATA {
        return Ok(());
    }

    bot.answer_callback_query(&query.id).await?;

    let target_chat = query
        .message
        .as_ref()
        .map(|message| message.chat.id)
        .unwrap_or_else(|| {
            let id = i64::try_from(query.from.id.0).unwrap_or_default();
            ChatId(id)
        });

    match check_subscriptions(&bot, query.from.id).await? {
        SubscriptionState::Subscribed => {
            bot.send_message(target_chat, messages::guide_message())
                .await?;
        }
        SubscriptionState::NotSubscribed => {
            bot.send_message(target_chat, messages::NOT_SUBSCRIBED_MESSAGE)
                .reply_markup(ready_keyboard())
                .await?;
        }
        SubscriptionState::CannotVerify => {
            bot.send_message(target_chat, messages::CHECK_FAILED_MESSAGE)
                .reply_markup(ready_keyboard())
                .await?;
        }
    }

    Ok(())
}

fn ready_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        messages::DONE_BUTTON_TEXT,
        messages::DONE_CALLBACK_DATA,
    )]])
}

enum SubscriptionState {
    Subscribed,
    NotSubscribed,
    CannotVerify,
}

async fn check_subscriptions(bot: &Bot, user_id: UserId) -> Result<SubscriptionState> {
    for channel in messages::required_channels() {
        let recipient = Recipient::from(channel.clone());
        let member = match bot.get_chat_member(recipient, user_id).await {
            Ok(member) => member,
            Err(err) => {
                warn!("Не удалось проверить подписку для {}: {:?}", channel, err);
                return Ok(SubscriptionState::CannotVerify);
            }
        };

        let is_member = match member.kind {
            ChatMemberKind::Owner(_)
            | ChatMemberKind::Administrator(_)
            | ChatMemberKind::Member => true,
            ChatMemberKind::Restricted(ref restricted) => restricted.is_member,
            ChatMemberKind::Left | ChatMemberKind::Banned(_) => false,
        };

        if !is_member {
            return Ok(SubscriptionState::NotSubscribed);
        }
    }

    Ok(SubscriptionState::Subscribed)
}
