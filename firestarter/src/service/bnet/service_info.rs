#![allow(missing_docs)]

//! BNet services - known information
//!
//! The response service is both imported and exported!
//! 3625566374   - bnet.protocol.ResponseService
//!
//! Client imported (from server)
//! The client asks us which services map to which ID. The services below
//! are listed in asking order. We can just return a direct mapping back
//! so the services are locked to the order asked by the client.
//!
//! 0-   1698982289  - bnet.protocol.connection.ConnectionService
//! 1-   1658456209  - bnet.protocol.account.AccountService
//! 2-   1128824125  - bnet.protocol.achievements.AchievementsService
//! 3-   233634817   - bnet.protocol.authentication.AuthenticationServer
//! 4-   3686756121  - bnet.protocol.challenge.ChallengeService
//! 5-   2198078984  - bnet.protocol.channel_invitation.ChannelInvitationService
//! 6-   3073563442  - bnet.protocol.channel.Channel
//! 7-   101490829   - bnet.protocol.channel.ChannelOwner
//! 8-   3612349579  - bnet.protocol.exchange.ExchangeService
//! 9-   2749215165  - bnet.protocol.friends.FriendsService
//! 10-  2165092757  - bnet.protocol.game_master.GameMaster
//! 11-  1069623117  - bnet.protocol.game_utilities.GameUtilities
//! 12-  213793859   - bnet.protocol.notification.NotificationService
//! 13-  4194801407  - bnet.protocol.presence.PresenceService
//! 14-  2091868617  - bnet.protocol.report.ReportService
//! 15-  3971904954  - bnet.protocol.resources.Resources
//! 16-  170173073   - bnet.protocol.search.SearchService
//! 17-  1041835658  - bnet.protocol.user_manager.UserManagerService
//!
//! Client exported (from client)
//! The client tells us which services map to which ID. The following ID's are
//! absolute, deviation is not allowed here.
//!
//! 1-   1423956503  - bnet.protocol.account.AccountNotify
//! 2-   3571241107  - bnet.protocol.achievements.AchievementsNotify
//! 3-   1898188341  - bnet.protocol.authentication.AuthenticationClient
//! 4-   3151632159  - bnet.protocol.challenge.ChallengeNotify
//! 5-   4035247136  - bnet.protocol.channel_invitation.ChannelInvitationNotify
//! 6-   3213656212  - bnet.protocol.channel.ChannelSubscriber
//! 7-   376431777   - bnet.protocol.exchange.ExchangeNotify
//! 8-   3111080599  - bnet.protocol.diag.DiagService
//! 9-   1864735251  - bnet.protocol.friends.FriendsNotify
//! 10-  3788189352  - bnet.protocol.notification.NotificationListener
//! 11-  3162975266  - bnet.protocol.user_manager.UserManagerNotify
//!
//! Unused services
//!
//! 689160787    - bnet.protocol.achievements.AchievementsUtils
//! 3338259653   - bnet.protocol.game_master.GameFactorySubscriber
//! 3826086206   - bnet.protocol.game_master.GameRequestSubscriber

use std::collections::HashMap;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ExportedServiceID {
    ConnectionService = 0,

    AccountService = 1,
    AchievementsService = 2,
    AuthenticationServer = 3,
    ChallengeService = 4,
    ChannelInvitationService = 5,
    Channel = 6,
    ChannelOwner = 7,
    ExchangeService = 8,
    FriendsService = 9,
    GameMaster = 10,
    GameUtilities = 11,
    NotificationService = 12,
    PresenceService = 13,
    ReportService = 14,
    Resources = 15,
    SearchService = 16,
    UserManagerService = 17,

    ResponseService = 254,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ImportedServiceID {
    AccountNotify = 1,
    AchievementsNotify = 2,
    AuthenticationClient = 3,
    ChallengeNotify = 4,
    ChannelInvitationNotify = 5,
    ChannelSubscriber = 6,
    ExchangeNotify = 7,
    DiagService = 8,
    FriendsNotify = 9,
    NotificationListener = 10,
    UserManagerNotify = 11,

    ResponseService = 254,
}

lazy_static! {
    pub static ref SERVICES_EXPORTED_BINDING: HashMap<u32, ExportedServiceID> = {
        hashmap!{
            1698982289 => ExportedServiceID::ConnectionService,

            1658456209 => ExportedServiceID::AccountService,
            1128824125 => ExportedServiceID::AchievementsService,
            233634817 => ExportedServiceID::AuthenticationServer,
            3686756121 => ExportedServiceID::ChallengeService,
            2198078984 => ExportedServiceID::ChannelInvitationService,
            3073563442 => ExportedServiceID::Channel,
            101490829 => ExportedServiceID::ChannelOwner,
            3612349579 => ExportedServiceID::ExchangeService,
            2749215165 => ExportedServiceID::FriendsService,
            2165092757 => ExportedServiceID::GameMaster,
            1069623117 => ExportedServiceID::GameUtilities,
            213793859 => ExportedServiceID::NotificationService,
            4194801407 => ExportedServiceID::PresenceService,
            2091868617 => ExportedServiceID::ReportService,
            3971904954 => ExportedServiceID::Resources,
            170173073 => ExportedServiceID::SearchService,
            1041835658 => ExportedServiceID::UserManagerService,

            3625566374 => ExportedServiceID::ResponseService,
        }
    };
    pub static ref SERVICES_IMPORTED_BINDING: HashMap<u32, ImportedServiceID> = {
        hashmap!{
            1423956503 =>  ImportedServiceID::AccountNotify,
            3571241107 =>  ImportedServiceID::AchievementsNotify,
            1898188341 =>  ImportedServiceID::AuthenticationClient,
            3151632159 =>  ImportedServiceID::ChallengeNotify,
            4035247136 =>  ImportedServiceID::ChannelInvitationNotify,
            3213656212 =>  ImportedServiceID::ChannelSubscriber,
            376431777 =>  ImportedServiceID::ExchangeNotify,
            3111080599 =>  ImportedServiceID::DiagService,
            1864735251 =>  ImportedServiceID::FriendsNotify,
            3788189352 => ImportedServiceID::NotificationListener,
            3162975266 => ImportedServiceID::UserManagerNotify,

            3625566374 => ImportedServiceID::ResponseService,
        }
    };
}
