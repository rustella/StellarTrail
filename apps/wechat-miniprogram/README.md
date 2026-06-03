# WeChat Mini Program Local Config

The tracked `project.config.json` must keep the safe placeholder `touristappid`.
Do not commit a real WeChat AppID, AppSecret, private key, or developer-local
tool setting.

For local development in WeChat Developer Tools, put the real AppID in the
ignored `project.private.config.json` file:

```json
{
  "appid": "your-local-wechat-appid"
}
```

`project.private.config.json` is intentionally ignored by Git so each developer
can keep their own local Mini Program project settings without exposing them in
repository history.

For API request signing, copy `miniprogram/config.example.ts` to the ignored
`miniprogram/config.ts` file and replace only the local copy's
`requestSignature.app_id` and `requestSignature.app_secret` values. The tracked
example must keep placeholder values.
