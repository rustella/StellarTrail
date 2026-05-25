import Foundation

struct AppLogger {
    static func info(_ message: String) {
        #if DEBUG
        print("[StellarTrail] \(message)")
        #endif
    }

    static func error(_ message: String) {
        #if DEBUG
        print("[StellarTrail][error] \(message)")
        #endif
    }
}
