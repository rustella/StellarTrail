import Foundation
import Combine

@MainActor
final class SessionStore: ObservableObject {
    @Published private(set) var currentSession: Session?

    private let keychainStore: SecureTokenStoring
    private let key = "stellartrail.session"

    init(keychainStore: SecureTokenStoring) {
        self.keychainStore = keychainStore
        self.currentSession = try? keychainStore.data(for: key).flatMap { try JSONDecoder.stellarTrail.decode(Session.self, from: $0) }
    }

    var isLoggedIn: Bool { currentSession != nil }

    func replace(with response: LoginResponse) {
        replace(with: Session(response: response))
    }

    func replace(with session: Session) {
        currentSession = session
        do {
            let data = try JSONEncoder.stellarTrail.encode(session)
            try keychainStore.set(data, for: key)
        } catch {
            AppLogger.error("Failed to persist session")
        }
    }

    func clear() {
        currentSession = nil
        do { try keychainStore.remove(key) } catch { AppLogger.error("Failed to clear session") }
    }
}
