# ベンチマーク

Pythonベンチマークは、Pythonのスケジューリング、Rustバインディング、JSONシリアライズ、ファイルI/O、SQLiteメタデータ更新を含む、async書き込み・読み込みのエンドツーエンド性能を測定します。

```bash
python benchmark/python/async_throughput.py --items 1000 --concurrency 32
```

変更を比較する際は、同一マシン・同一ストレージ種別で実行してください。結果は普遍的な性能主張ではなく、性能劣化を見つけるための指標として扱います。

English guide: [README.md](README.md)
