import Foundation

private let apiPrefix = "/api/v1"
private let healthPath = "/healthz"

enum HTTPMethod: String {
    case get = "GET"
    case post = "POST"
    case put = "PUT"
    case patch = "PATCH"
    case delete = "DELETE"
}

struct APIRequest {
    let method: HTTPMethod
    let path: String
    let queryItems: [URLQueryItem]
    let body: Data?

    static func get(_ path: String, queryItems: [URLQueryItem] = []) -> APIRequest {
        APIRequest(method: .get, path: path, queryItems: queryItems, body: nil)
    }

    static func post<Body: Encodable>(_ path: String, body: Body) throws -> APIRequest {
        APIRequest(method: .post, path: path, queryItems: [], body: try JSONEncoder.stellarTrail.encode(body))
    }

    static func post(_ path: String) -> APIRequest {
        APIRequest(method: .post, path: path, queryItems: [], body: nil)
    }

    static func put<Body: Encodable>(_ path: String, body: Body) throws -> APIRequest {
        APIRequest(method: .put, path: path, queryItems: [], body: try JSONEncoder.stellarTrail.encode(body))
    }

    static func patch<Body: Encodable>(_ path: String, body: Body) throws -> APIRequest {
        APIRequest(method: .patch, path: path, queryItems: [], body: try JSONEncoder.stellarTrail.encode(body))
    }

    static func delete(_ path: String) -> APIRequest {
        APIRequest(method: .delete, path: path, queryItems: [], body: nil)
    }
}

@MainActor
final class APIClient {
    private let settingsStore: AppSettingsStore
    private let sessionStore: SessionStore
    private let session: URLSession

    init(settingsStore: AppSettingsStore, sessionStore: SessionStore, session: URLSession = .shared) {
        self.settingsStore = settingsStore
        self.sessionStore = sessionStore
        self.session = session
    }

    func send<T: Decodable>(_ request: APIRequest, requiresAuth: Bool, retryOnUnauthorized: Bool = true) async throws -> T {
        let data = try await perform(request, requiresAuth: requiresAuth, retryOnUnauthorized: retryOnUnauthorized)
        if T.self == EmptyResponse.self && data.isEmpty {
            return EmptyResponse() as! T
        }
        do {
            return try JSONDecoder.stellarTrail.decode(T.self, from: data)
        } catch {
            throw AppError.decoding(error.localizedDescription)
        }
    }

    @discardableResult
    func sendEmpty(_ request: APIRequest, requiresAuth: Bool, retryOnUnauthorized: Bool = true) async throws -> EmptyResponse {
        try await send(request, requiresAuth: requiresAuth, retryOnUnauthorized: retryOnUnauthorized)
    }

    func uploadAvatar(data: Data, fileName: String = "avatar.jpg", mimeType: String = "image/jpeg", retryOnUnauthorized: Bool = true) async throws -> ProfileUserResponse {
        let responseData = try await uploadAvatarData(data: data, fileName: fileName, mimeType: mimeType, retryOnUnauthorized: retryOnUnauthorized)
        do {
            return try JSONDecoder.stellarTrail.decode(ProfileUserResponse.self, from: responseData)
        } catch {
            throw AppError.decoding(error.localizedDescription)
        }
    }

    private func uploadAvatarData(data: Data, fileName: String, mimeType: String, retryOnUnauthorized: Bool) async throws -> Data {
        let boundary = "Boundary-\(UUID().uuidString)"
        let url = try buildURL(path: "/me/profile/avatar", queryItems: [])
        var request = URLRequest(url: url)
        request.httpMethod = HTTPMethod.put.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        request.setValue("zh-CN", forHTTPHeaderField: "X-StellarTrail-Locale")
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")
        guard let token = sessionStore.currentSession?.accessToken else { throw AppError.missingSession }
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.httpBody = Self.multipartBody(data: data, fieldName: "file", fileName: fileName, mimeType: mimeType, boundary: boundary)

        return try await performURLRequest(request, requiresAuth: true, retryOnUnauthorized: retryOnUnauthorized) {
            try await self.uploadAvatarData(data: data, fileName: fileName, mimeType: mimeType, retryOnUnauthorized: false)
        }
    }

    private func perform(_ request: APIRequest, requiresAuth: Bool, retryOnUnauthorized: Bool) async throws -> Data {
        let urlRequest = try buildURLRequest(from: request, requiresAuth: requiresAuth)
        return try await performURLRequest(urlRequest, requiresAuth: requiresAuth, retryOnUnauthorized: retryOnUnauthorized) {
            try await self.perform(request, requiresAuth: requiresAuth, retryOnUnauthorized: false)
        }
    }

    private func performURLRequest(_ urlRequest: URLRequest, requiresAuth: Bool, retryOnUnauthorized: Bool, retry: @escaping () async throws -> Data) async throws -> Data {
        do {
            let (data, response) = try await session.data(for: urlRequest)
            guard let http = response as? HTTPURLResponse else { throw AppError.network("响应无效") }
            if http.statusCode == 401, requiresAuth, retryOnUnauthorized {
                try await refreshSession()
                return try await retry()
            }
            guard (200..<300).contains(http.statusCode) else {
                throw decodeError(data: data, statusCode: http.statusCode)
            }
            return data
        } catch let error as AppError {
            throw error
        } catch {
            throw AppError.network(error.localizedDescription)
        }
    }

    private func buildURLRequest(from request: APIRequest, requiresAuth: Bool) throws -> URLRequest {
        let url = try buildURL(path: request.path, queryItems: request.queryItems)
        var urlRequest = URLRequest(url: url)
        urlRequest.httpMethod = request.method.rawValue
        urlRequest.setValue("application/json", forHTTPHeaderField: "Accept")
        urlRequest.setValue("zh-CN", forHTTPHeaderField: "X-StellarTrail-Locale")
        if let body = request.body {
            urlRequest.httpBody = body
            urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        }
        if requiresAuth {
            guard let token = sessionStore.currentSession?.accessToken else { throw AppError.missingSession }
            urlRequest.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        return urlRequest
    }

    func resolveAssetURL(_ pathOrURL: String) -> URL? {
        if let absoluteURL = URL(string: pathOrURL), absoluteURL.scheme == "http" || absoluteURL.scheme == "https" {
            return absoluteURL
        }
        guard var components = URLComponents(url: settingsStore.assetsBaseURL, resolvingAgainstBaseURL: false) else {
            return nil
        }
        let basePath = components.path.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        let assetPath = pathOrURL.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        components.path = "/" + [basePath, assetPath].filter { !$0.isEmpty }.joined(separator: "/")
        return components.url
    }

    private func buildURL(path: String, queryItems: [URLQueryItem]) throws -> URL {
        guard var components = URLComponents(url: settingsStore.baseURL, resolvingAgainstBaseURL: false) else {
            throw AppError.invalidURL
        }
        let basePath = components.path.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        let requestPath = versionedAPIPath(path).trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        components.path = "/" + [basePath, requestPath].filter { !$0.isEmpty }.joined(separator: "/")
        components.queryItems = queryItems.isEmpty ? nil : queryItems
        guard let url = components.url else { throw AppError.invalidURL }
        return url
    }

    private func refreshSession() async throws {
        guard let refreshToken = sessionStore.currentSession?.refreshToken else {
            sessionStore.clear()
            throw AppError.unauthorized
        }
        let body = RefreshTokenRequest(refreshToken: refreshToken)
        let response: LoginResponse = try await send(try APIRequest.post("/auth/refresh", body: body), requiresAuth: false, retryOnUnauthorized: false)
        sessionStore.replace(with: response)
    }

    private func decodeError(data: Data, statusCode: Int) -> AppError {
        if let envelope = try? JSONDecoder.stellarTrail.decode(APIErrorEnvelope.self, from: data) {
            if envelope.code == "captcha_required" {
                return .captchaRequired(envelope.message ?? "请先完成验证码验证")
            }
            if let message = envelope.message?.nilIfBlank {
                return .server(message)
            }
        }
        if statusCode == 401 { return .unauthorized }
        return .server("请求失败（\(statusCode)）")
    }

    private static func multipartBody(data: Data, fieldName: String, fileName: String, mimeType: String, boundary: String) -> Data {
        var body = Data()
        body.appendString("--\(boundary)\r\n")
        body.appendString("Content-Disposition: form-data; name=\"\(fieldName)\"; filename=\"\(fileName)\"\r\n")
        body.appendString("Content-Type: \(mimeType)\r\n\r\n")
        body.append(data)
        body.appendString("\r\n--\(boundary)--\r\n")
        return body
    }
}

private func versionedAPIPath(_ path: String) -> String {
    if path == healthPath || path.hasPrefix("\(apiPrefix)/") {
        return path
    }
    let normalized = path.hasPrefix("/") ? path : "/\(path)"
    return apiPrefix + normalized
}

private extension Data {
    mutating func appendString(_ string: String) {
        append(Data(string.utf8))
    }
}
