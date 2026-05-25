import Foundation

struct ClientConfig: Equatable {
    let apiBaseURLString: String
    let assetsBaseURLString: String
    let domainCandidates: [ClientDomainCandidate]

    init(
        apiBaseURLString: String,
        assetsBaseURLString: String,
        domainCandidates: [ClientDomainCandidate] = []
    ) {
        self.apiBaseURLString = apiBaseURLString
        self.assetsBaseURLString = assetsBaseURLString
        self.domainCandidates = domainCandidates
    }

    static let production = ClientConfig(
        apiBaseURLString: "https://api.example.invalid",
        assetsBaseURLString: "https://assets.example.invalid"
    )

    static func load(from bundle: Bundle = .main) -> ClientConfig {
        guard let url = bundle.url(forResource: "ClientConfig", withExtension: "plist"),
              let data = try? Data(contentsOf: url),
              let raw = try? PropertyListDecoder().decode(RawClientConfig.self, from: data) else {
            return production
        }
        return ClientConfig(
            apiBaseURLString: sanitizeAPIBaseURL(raw.apiBaseURLString, fallback: production.apiBaseURLString),
            assetsBaseURLString: sanitizeBaseURL(raw.assetsBaseURLString, fallback: production.assetsBaseURLString),
            domainCandidates: normalizeDomainCandidates(raw.domainCandidates)
        )
    }

    static func sanitizeAPIBaseURL(_ value: String?, fallback: String) -> String {
        sanitizeBaseURL(value, fallback: fallback)
    }

    static func sanitizeBaseURL(_ value: String?, fallback: String) -> String {
        guard let trimmed = value?.trimmingCharacters(in: .whitespacesAndNewlines), !trimmed.isEmpty else {
            return fallback
        }
        return String(trimmed.dropTrailingSlashes())
    }

    private static func normalizeDomainCandidates(_ candidates: [RawClientDomainCandidate]?) -> [ClientDomainCandidate] {
        candidates?.compactMap { candidate in
            let id = candidate.id?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            let apiBaseURLString = sanitizeBaseURL(candidate.apiBaseURLString, fallback: "")
            let assetsBaseURLString = sanitizeBaseURL(candidate.assetsBaseURLString, fallback: "")
            guard !id.isEmpty, !apiBaseURLString.isEmpty, !assetsBaseURLString.isEmpty else {
                return nil
            }
            return ClientDomainCandidate(
                id: id,
                apiBaseURLString: apiBaseURLString,
                assetsBaseURLString: assetsBaseURLString
            )
        } ?? []
    }
}

struct ClientDomainCandidate: Equatable {
    let id: String
    let apiBaseURLString: String
    let assetsBaseURLString: String
}

private struct RawClientConfig: Decodable {
    let apiBaseURLString: String?
    let assetsBaseURLString: String?
    let domainCandidates: [RawClientDomainCandidate]?

    enum CodingKeys: String, CodingKey {
        case apiBaseURLString = "API_BASE_URL"
        case assetsBaseURLString = "ASSETS_BASE_URL"
        case domainCandidates = "DOMAIN_CANDIDATES"
    }
}

private struct RawClientDomainCandidate: Decodable {
    let id: String?
    let apiBaseURLString: String?
    let assetsBaseURLString: String?

    enum CodingKeys: String, CodingKey {
        case id = "ID"
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
