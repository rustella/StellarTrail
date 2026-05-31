import Foundation

enum ClientKey: String, Codable, CaseIterable, Identifiable {
    case wechatMiniprogram = "wechat_miniprogram"
    case web
    case android
    case ios
    case macos

    var id: String { rawValue }
}

enum ClientVersionStatus: String, Codable, CaseIterable, Identifiable {
    case draft
    case published

    var id: String { rawValue }
}

struct ClientVersionReleaseNoteSection: Codable, Equatable, Identifiable {
    let key: String
    let title: String
    let items: [String]

    var id: String { key }
}

struct ClientVersion: Codable, Equatable, Identifiable {
    let id: String
    let clientKey: ClientKey
    let version: String
    let title: String
    let releaseNotes: [String]
    let releaseNoteSections: [ClientVersionReleaseNoteSection]
    let status: ClientVersionStatus
    let publishedAt: String?
    let createdAt: String
    let updatedAt: String

    enum CodingKeys: String, CodingKey {
        case id, clientKey, version, title, releaseNotes, releaseNoteSections, status, publishedAt, createdAt, updatedAt
    }

    init(
        id: String,
        clientKey: ClientKey,
        version: String,
        title: String,
        releaseNotes: [String] = [],
        releaseNoteSections: [ClientVersionReleaseNoteSection] = [],
        status: ClientVersionStatus = .published,
        publishedAt: String? = nil,
        createdAt: String,
        updatedAt: String
    ) {
        self.id = id
        self.clientKey = clientKey
        self.version = version
        self.title = title
        self.releaseNotes = releaseNotes
        self.releaseNoteSections = releaseNoteSections
        self.status = status
        self.publishedAt = publishedAt
        self.createdAt = createdAt
        self.updatedAt = updatedAt
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(String.self, forKey: .id)
        clientKey = try container.decode(ClientKey.self, forKey: .clientKey)
        version = try container.decode(String.self, forKey: .version)
        title = try container.decode(String.self, forKey: .title)
        releaseNotes = try container.decodeIfPresent([String].self, forKey: .releaseNotes) ?? []
        releaseNoteSections = try container.decodeIfPresent([ClientVersionReleaseNoteSection].self, forKey: .releaseNoteSections) ?? []
        status = try container.decode(ClientVersionStatus.self, forKey: .status)
        publishedAt = try container.decodeIfPresent(String.self, forKey: .publishedAt)
        createdAt = try container.decode(String.self, forKey: .createdAt)
        updatedAt = try container.decode(String.self, forKey: .updatedAt)
    }
}

struct ListClientVersionsRequest: Equatable {
    var clientKey: ClientKey = .ios
    var limit: Int = 20
    var cursor: String?

    var queryItems: [URLQueryItem] {
        var items = [
            URLQueryItem(name: "client_key", value: clientKey.rawValue),
            URLQueryItem(name: "limit", value: String(limit))
        ]
        if let cursor { items.append(URLQueryItem(name: "cursor", value: cursor)) }
        return items
    }
}

struct ListClientVersionsResponse: Codable, Equatable {
    let items: [ClientVersion]
    let nextCursor: String?
}

enum RoadmapStatus: String, Codable, CaseIterable, Identifiable {
    case planned
    case designing
    case building
    case preview
    case shipped

    var id: String { rawValue }

    var label: String {
        switch self {
        case .planned: return "规划中"
        case .designing: return "设计中"
        case .building: return "开发中"
        case .preview: return "预览"
        case .shipped: return "已上线"
        }
    }
}

enum RoadmapCategory: String, Codable, CaseIterable, Identifiable {
    case gear
    case skills
    case routes
    case offline
    case safety
    case community

    var id: String { rawValue }

    var label: String {
        switch self {
        case .gear: return "装备"
        case .skills: return "技能"
        case .routes: return "路线"
        case .offline: return "离线"
        case .safety: return "安全"
        case .community: return "社区"
        }
    }
}

struct RoadmapItem: Codable, Equatable, Identifiable {
    let id: String
    let clientKey: ClientKey
    let title: String
    let summary: String
    let details: String?
    let category: String
    let status: String
    let priority: Int
    let sortOrder: Int
    let isPublished: Bool
    let voteCount: Int
    let subscriptionCount: Int
    let isVoted: Bool
    let isSubscribed: Bool
    let publishedAt: String?
    let createdAt: String
    let updatedAt: String

    var categoryLabel: String {
        RoadmapCategory(rawValue: category)?.label ?? category
    }

    var statusLabel: String {
        RoadmapStatus(rawValue: status)?.label ?? status
    }
}

struct ListRoadmapRequest: Equatable {
    var clientKey: ClientKey = .ios
    var status: RoadmapStatus?
    var limit: Int = 50
    var cursor: String?

    var queryItems: [URLQueryItem] {
        var items = [
            URLQueryItem(name: "client_key", value: clientKey.rawValue),
            URLQueryItem(name: "limit", value: String(limit))
        ]
        if let status { items.append(URLQueryItem(name: "status", value: status.rawValue)) }
        if let cursor { items.append(URLQueryItem(name: "cursor", value: cursor)) }
        return items
    }
}

struct ListRoadmapResponse: Codable, Equatable {
    let items: [RoadmapItem]
    let nextCursor: String?
}

struct UploadImageInfo: Codable, Equatable, Identifiable {
    let id: String
    let purpose: String
    let originalFilename: String
    let imageType: String
    let contentType: String
    let sizeBytes: Int
    let sha256: String
    let downloadUrl: String
    let isDeleted: Bool
    let createdAt: String
}

enum FeedbackCategory: String, Codable, CaseIterable, Identifiable {
    case bug
    case feature
    case content
    case other

    var id: String { rawValue }

    var label: String {
        switch self {
        case .bug: return "问题反馈"
        case .feature: return "功能建议"
        case .content: return "内容纠错"
        case .other: return "其他"
        }
    }
}

struct CreateFeedbackRequest: Codable, Equatable {
    var category: String
    var content: String
    var contact: String?
    var page: String?
    var clientPlatform: String?
    var clientVersion: String?
    var deviceModel: String?
    var imageIds: [String]
}

struct FeedbackResponse: Codable, Equatable, Identifiable {
    let id: String
    let category: String
    let content: String
    let contact: String?
    let page: String?
    let clientPlatform: String?
    let clientVersion: String?
    let deviceModel: String?
    let status: String
    let images: [UploadImageInfo]
    let isDeleted: Bool
    let createdAt: String
    let updatedAt: String
}

struct KnotDisclaimerResponse: Codable, Equatable {
    let key: String
    let version: String
    let title: String
    let content: String
    let accepted: Bool
    let acceptedAt: String?
}

struct AcceptKnotDisclaimerRequest: Codable, Equatable {
    var clientPlatform: String? = "ios"
    var clientVersion: String? = nil
    var deviceModel: String? = nil
}
