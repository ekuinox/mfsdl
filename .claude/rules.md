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

## Code Quality

### Rust ファイル編集後の必須チェック

`.rs` ファイルを編集した際は、必ず以下のコマンドを実行してコードを整理してください。

```bash
cargo check   # コンパイルチェック
cargo fmt     # コードフォーマット
cargo clippy  # Lint チェック
```

### 実行順序

1. `cargo check` - コンパイルエラーがないか確認
2. `cargo fmt` - コードを自動フォーマット
3. `cargo clippy` - Lint 警告を確認し、必要に応じて修正

### 注意点

- clippy の警告は可能な限り修正すること
- フォーマットの変更は自動的に適用されるため、確認は不要
