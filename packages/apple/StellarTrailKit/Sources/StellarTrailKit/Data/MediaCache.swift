import Foundation

@MainActor
protocol MediaCacheManaging {
    func resolvedURL(for remoteURL: URL) -> URL
    func cache(urls: [URL], estimatedBytes: Int, progress: @escaping (MediaCacheProgress) -> Void) async -> MediaCacheResult
}

@MainActor
final class MediaCacheManager: MediaCacheManaging {
    private let fileManager: FileManager
    private let session: URLSession
    private let cacheDirectory: URL

    init(fileManager: FileManager = .default, session: URLSession = .shared) {
        self.fileManager = fileManager
        self.session = session
        let root = fileManager.urls(for: .cachesDirectory, in: .userDomainMask).first ?? URL(fileURLWithPath: NSTemporaryDirectory())
        self.cacheDirectory = root.appendingPathComponent("StellarTrailMediaCache", isDirectory: true)
        try? fileManager.createDirectory(at: cacheDirectory, withIntermediateDirectories: true)
    }

    func resolvedURL(for remoteURL: URL) -> URL {
        let local = cacheFileURL(for: remoteURL)
        return fileManager.fileExists(atPath: local.path) ? local : remoteURL
    }

    func cache(urls: [URL], estimatedBytes: Int, progress: @escaping (MediaCacheProgress) -> Void) async -> MediaCacheResult {
        let uniqueURLs = Array(Set(urls)).sorted { $0.absoluteString < $1.absoluteString }
        guard !uniqueURLs.isEmpty else {
            progress(MediaCacheProgress(completed: 0, total: 0))
            return MediaCacheResult(completed: 0, total: 0, failed: 0, estimatedBytes: estimatedBytes)
        }

        var completed = 0
        var failed = 0
        progress(MediaCacheProgress(completed: completed, total: uniqueURLs.count))

        for remoteURL in uniqueURLs {
            let localURL = cacheFileURL(for: remoteURL)
            if !fileManager.fileExists(atPath: localURL.path) {
                do {
                    let (temporaryURL, _) = try await session.download(from: remoteURL)
                    try? fileManager.removeItem(at: localURL)
                    try fileManager.moveItem(at: temporaryURL, to: localURL)
                } catch {
                    failed += 1
                }
            }
            completed += 1
            progress(MediaCacheProgress(completed: completed, total: uniqueURLs.count))
        }

        return MediaCacheResult(completed: completed - failed, total: uniqueURLs.count, failed: failed, estimatedBytes: estimatedBytes)
    }

    private func cacheFileURL(for remoteURL: URL) -> URL {
        let name = remoteURL.absoluteString
            .data(using: .utf8)?
            .base64EncodedString()
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "=", with: "")
            ?? UUID().uuidString
        let pathExtension = remoteURL.pathExtension.nilIfBlank ?? "asset"
        return cacheDirectory.appendingPathComponent(name).appendingPathExtension(pathExtension)
    }
}

@MainActor
final class FixtureMediaCacheManager: MediaCacheManaging {
    func resolvedURL(for remoteURL: URL) -> URL { remoteURL }

    func cache(urls: [URL], estimatedBytes: Int, progress: @escaping (MediaCacheProgress) -> Void) async -> MediaCacheResult {
        let total = Set(urls).count
        progress(MediaCacheProgress(completed: total, total: total))
        return MediaCacheResult(completed: total, total: total, failed: 0, estimatedBytes: estimatedBytes)
    }
}
