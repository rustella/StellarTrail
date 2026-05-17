import Foundation
import Combine

enum AuthMode: String, CaseIterable, Identifiable {
    case login
    case register

    var id: String { rawValue }
    var title: String { self == .login ? "账号登录" : "注册并登录" }
}

@MainActor
final class AuthViewModel: ObservableObject {
    @Published var mode: AuthMode
    @Published var account = ""
    @Published var password = ""
    @Published var username = ""
    @Published var email = ""
    @Published var confirmPassword = ""
    @Published var verificationCode = ""
    @Published var captchaAnswer = ""
    @Published private(set) var captchaTicket: String?
    @Published private(set) var captchaSvg: String?
    @Published private(set) var debugCode: String?
    @Published private(set) var loading = false
    @Published private(set) var message: String?
    @Published private(set) var completed = false

    private let repository: any AuthRepositorying
    private let sessionStore: SessionStore

    init(mode: AuthMode = .login, repository: any AuthRepositorying, sessionStore: SessionStore) {
        self.mode = mode
        self.repository = repository
        self.sessionStore = sessionStore
    }

    var canSubmit: Bool {
        switch mode {
        case .login:
            return account.nilIfBlank != nil && password.nilIfBlank != nil
        case .register:
            return username.nilIfBlank != nil && email.nilIfBlank != nil && password.nilIfBlank != nil && confirmPassword == password && verificationCode.nilIfBlank != nil
        }
    }

    func sendVerificationCode() async {
        guard let email = email.nilIfBlank else {
            message = "请先填写邮箱"
            return
        }
        loading = true
        defer { loading = false }
        do {
            let response = try await repository.sendEmailVerificationCode(email: email)
            debugCode = response.debugCode
            message = response.debugCode.map { "验证码已发送，本地调试码：\($0)" } ?? "验证码已发送，请查看邮箱"
        } catch {
            message = error.localizedDescription
        }
    }

    func requestCaptcha() async {
        guard let account = account.nilIfBlank else {
            message = "请先填写账号"
            return
        }
        loading = true
        defer { loading = false }
        do {
            let response = try await repository.captcha(account: account)
            captchaTicket = response.captchaTicket
            captchaSvg = response.imageSvg
            message = response.debugAnswer.map { "请完成验证码，本地调试答案：\($0)" } ?? "请完成验证码后继续"
        } catch {
            message = error.localizedDescription
        }
    }

    func submit() async {
        guard canSubmit else {
            message = "请补全必填信息"
            return
        }
        loading = true
        defer { loading = false }
        do {
            let response: LoginResponse
            switch mode {
            case .login:
                response = try await repository.login(account: account, password: password, captchaTicket: captchaTicket, captchaAnswer: captchaAnswer.nilIfBlank)
            case .register:
                let request = RegisterRequest(username: username, email: email, password: password, confirmPassword: confirmPassword, emailVerificationCode: verificationCode)
                response = try await repository.register(request)
            }
            sessionStore.replace(with: response)
            completed = true
        } catch AppError.captchaRequired(let text) {
            message = text
            await requestCaptcha()
        } catch {
            message = error.localizedDescription
        }
    }
}
