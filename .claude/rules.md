# Development Rules

このプロジェクトで Claude が従うべきルールを記載します。

## Rust Dependencies

### クレートの追加

Cargo.toml を直接編集せず、必ず `cargo add` コマンドを使用してください。

```bash
# ✅ Good
cargo add indicatif

# ❌ Bad
# Cargo.toml を直接編集
```

### 理由

- Cargo.lock の自動更新
- バージョン解決の自動化
- 依存関係の整合性保証
