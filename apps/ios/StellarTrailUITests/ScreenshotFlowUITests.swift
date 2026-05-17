import XCTest

final class ScreenshotFlowUITests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    func testReviewScreenshotFlow() throws {
        let app = XCUIApplication()
        app.launchArguments += ["--stellartrail-screenshot-fixtures"]
        app.launchEnvironment["STELLARTRAIL_SCREENSHOT_MODE"] = "1"
        app.launch()

        capture("01-home-guest-light", app: app)

        app.tabBars.buttons["装备"].tap()
        capture("05-gears-guest-light", app: app)

        app.buttons["账号登录"].firstMatch.tap()
        capture("02-login-light", app: app)
        app.buttons["注册账号"].firstMatch.tap()
        capture("03-register-light", app: app)
        app.buttons["账号登录"].firstMatch.tap()
        signIn(app)

        app.tabBars.buttons["首页"].tap()
        capture("04-home-signed-in-light", app: app)

        app.tabBars.buttons["装备"].tap()
        capture("06-gears-available-light", app: app)
        app.buttons["历史"].firstMatch.tap()
        capture("07-gears-history-light", app: app)
        app.buttons["可用"].firstMatch.tap()
        app.buttons["轻量背包"].firstMatch.tap()
        capture("08-gear-detail-light", app: app)
        app.buttons["编辑装备"].firstMatch.tap()
        capture("11-gear-edit-light", app: app)
        app.buttons["关闭"].firstMatch.tap()
        app.navigationBars.buttons.element(boundBy: 0).tap()
        app.buttons["添加装备"].firstMatch.tap()
        capture("09-gear-create-top-light", app: app)
        app.swipeUp()
        capture("10-gear-create-bottom-light", app: app)
        app.buttons["关闭"].firstMatch.tap()

        app.tabBars.buttons["技能"].tap()
        capture("12-skills-light", app: app)
        app.buttons["单套结"].firstMatch.tap()
        capture("13-knot-detail-light", app: app)
        app.buttons["关闭"].firstMatch.tap()

        app.tabBars.buttons["我的"].tap()
        capture("15-profile-signed-in-light", app: app)
    }

    private func signIn(_ app: XCUIApplication) {
        let account = app.textFields["用户名或邮箱"]
        if account.waitForExistence(timeout: 3) {
            account.tap()
            account.typeText("trail_alice")
        }
        let password = app.secureTextFields["密码"]
        if password.waitForExistence(timeout: 3) {
            password.tap()
            password.typeText("OutdoorPass123!")
        }
        app.buttons["账号登录"].lastMatch.tap()
        XCTAssertTrue(app.tabBars.buttons["装备"].waitForExistence(timeout: 5))
    }

    private func capture(_ name: String, app: XCUIApplication) {
        let attachment = XCTAttachment(screenshot: app.screenshot())
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
