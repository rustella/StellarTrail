import Foundation

enum AppError: Error, Equatable, LocalizedError {
    case invalidURL
    case missingSession
    case unauthorized
    case captchaRequired(String)
    case server(String)
    case decoding(String)
    case network(String)
    case unknown

    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return "连接地址不正确"
        case .missingSession:
            return "请先登录后再继续"
        case .unauthorized:
            return "登录已过期，请重新登录"
        case let .captchaRequired(message):
            return message
        case let .server(message):
            return message
        case let .decoding(message):
            return "数据暂时无法显示：\(message)"
        case let .network(message):
            return "暂时连不上，请稍后再试：\(message)"
        case .unknown:
            return "暂时无法完成，请稍后再试"
        }
    }
}

struct APIErrorEnvelope: Decodable {
    let code: String?
    let message: String?
}
