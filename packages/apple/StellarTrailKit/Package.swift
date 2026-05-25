// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "StellarTrailKit",
    defaultLocalization: "zh-Hans",
    platforms: [
        .iOS(.v17),
        .macOS(.v14)
    ],
    products: [
        .library(name: "StellarTrailKit", targets: ["StellarTrailKit"])
    ],
    targets: [
        .target(
            name: "StellarTrailKit",
            path: "Sources/StellarTrailKit"
        ),
        .testTarget(
            name: "StellarTrailKitTests",
            dependencies: ["StellarTrailKit"],
            path: "Tests/StellarTrailKitTests"
        )
    ]
)
