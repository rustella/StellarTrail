import Foundation

struct ClientConfig: Equatable {
    let apiBaseURLString: String
    let assetsBaseURLString: String
    let clientIdentity: String
    let domainCandidates: [ClientDomainCandidate]

    init(
        apiBaseURLString: String,
        assetsBaseURLString: String,
        clientIdentity: String = "ios/0.1.0",
        domainCandidates: [ClientDomainCandidate] = []
    ) {
        self.apiBaseURLString = apiBaseURLString
        self.assetsBaseURLString = assetsBaseURLString
        self.clientIdentity = clientIdentity
        self.domainCandidates = domainCandidates
    }

    static let production = makeProduction()

    static func makeProduction(client: String = "ios", version: String = "0.1.0") -> ClientConfig {
        ClientConfig(
            apiBaseURLString: "https://api.example.invalid",
            assetsBaseURLString: "https://assets.example.invalid",
            clientIdentity: buildClientIdentity(client: client, version: version)
        )
    }

    static func load(from bundle: Bundle = .main, client: String = "ios", version: String = "0.1.0") -> ClientConfig {
        let fallback = makeProduction(client: client, version: version)
        guard let url = bundle.url(forResource: "ClientConfig", withExtension: "plist"),
              let data = try? Data(contentsOf: url),
              let raw = try? PropertyListDecoder().decode(RawClientConfig.self, from: data) else {
            return fallback
        }
        return ClientConfig(
            apiBaseURLString: sanitizeAPIBaseURL(raw.apiBaseURLString, fallback: fallback.apiBaseURLString),
            assetsBaseURLString: sanitizeBaseURL(raw.assetsBaseURLString, fallback: fallback.assetsBaseURLString),
            clientIdentity: buildClientIdentity(
                client: raw.client,
                version: raw.clientVersion,
                fallbackClient: client,
                fallbackVersion: version
            ),
            domainCandidates: normalizeDomainCandidates(raw.domainCandidates)
        )
    }

    static func buildClientIdentity(
        client: String?,
        version: String?,
        fallbackClient: String = "ios",
        fallbackVersion: String = "0.1.0"
    ) -> String {
        let resolvedClient = sanitizeClientIdentityPart(client, fallback: fallbackClient, hardFallback: "ios")
        let resolvedVersion = sanitizeClientIdentityPart(version, fallback: fallbackVersion, hardFallback: "0.1.0")
        return "\(resolvedClient)/\(resolvedVersion)"
    }

    private static func sanitizeClientIdentityPart(_ value: String?, fallback: String, hardFallback: String) -> String {
        let trimmedFallback = fallback.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let trimmed = value?.trimmingCharacters(in: .whitespacesAndNewlines), !trimmed.isEmpty else {
            return trimmedFallback.isEmpty ? hardFallback : trimmedFallback
        }
        return trimmed
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
    let client: String?
    let clientVersion: String?
    let domainCandidates: [RawClientDomainCandidate]?

    enum CodingKeys: String, CodingKey {
        case apiBaseURLString = "API_BASE_URL"
        case assetsBaseURLString = "ASSETS_BASE_URL"
        case client = "CLIENT"
        case clientVersion = "CLIENT_VERSION"
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
