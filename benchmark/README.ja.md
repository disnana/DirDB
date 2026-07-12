# ベンチマーク

Pythonベンチマークは、Pythonのスケジューリング、Rustバインディング、構造化値変換、ファイルI/O、SQLiteメタデータ更新を含むエンドツーエンド性能を測定します。

### Asyncスループット

```bash
python benchmark/python/async_throughput.py --items 1000 --concurrency 32
```

### 辞書ラウンドトリップ

`db["path"] = value`と`db["path"]`をネストした辞書・リストで使います。PythonからRustへの構造化変換経路を測定します。

```bash
python benchmark/python/mapping_roundtrip.py --items 1000
```

変更を比較する際は、同一マシン・同一ストレージ種別で実行してください。結果は普遍的な性能主張ではなく、性能劣化を見つけるための指標として扱います。

English guide: [README.md](README.md)
