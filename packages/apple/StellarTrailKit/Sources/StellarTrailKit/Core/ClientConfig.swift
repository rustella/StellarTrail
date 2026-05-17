import Foundation

struct ClientConfig: Equatable {
    let apiBaseURLString: String
    let assetsBaseURLString: String

    static let production = ClientConfig(
        apiBaseURLString: "https://api.stellartrail.cn",
        assetsBaseURLString: "https://assets.stellartrail.cn"
    )

    static func load(from bundle: Bundle = .main) -> ClientConfig {
        guard let url = bundle.url(forResource: "ClientConfig", withExtension: "plist"),
              let data = try? Data(contentsOf: url),
              let raw = try? PropertyListDecoder().decode(RawClientConfig.self, from: data) else {
            return production
        }
        return ClientConfig(
            apiBaseURLString: sanitizeBaseURL(raw.apiBaseURLString, fallback: production.apiBaseURLString),
            assetsBaseURLString: sanitizeBaseURL(raw.assetsBaseURLString, fallback: production.assetsBaseURLString)
        )
    }

    private static func sanitizeBaseURL(_ value: String?, fallback: String) -> String {
        guard let trimmed = value?.trimmingCharacters(in: .whitespacesAndNewlines), !trimmed.isEmpty else {
            return fallback
        }
        return String(trimmed.dropTrailingSlashes())
    }
}

private struct RawClientConfig: Decodable {
    let apiBaseURLString: String?
    let assetsBaseURLString: String?

    enum CodingKeys: String, CodingKey {
        case apiBaseURLString = "API_BASE_URL"
        case assetsBaseURLString = "ASSETS_BASE_URL"
    }
}

private extension String {
    func dropTrailingSlashes() -> Substring {
        var end = endIndex
        while end > startIndex, self[index(before: end)] == "/" {
            end = index(before: end)
        }
        return self[..<end]
    }
}
