#!/usr/bin/env bash
# Menjalankan satu skenario antarmuka pada aplikasi sungguhan (build debug).
#   uji-ui/jalankan.sh uji-ui/<skenario>.js [folder-data]
# Butuh layar (DISPLAY). Hasil: $OUT/selesai.txt ("OK" atau "GAGAL: ...") dan potret-*.png.
set -uo pipefail
akar="$(cd "$(dirname "$0")/.." && pwd)"
skenario="$1"
nama="$(basename "$skenario" .js)"
OUT="${CATAT_UJI_HASIL:-$akar/.dev-data/uji-ui}/$nama"
DATA="${2:-$OUT/data}"
rm -rf "$OUT"; mkdir -p "$OUT"
[ -n "${2:-}" ] || mkdir -p "$DATA"
cat "$akar/uji-ui/pustaka.js" "$skenario" > "$OUT/skrip.js"
cat >> "$OUT/skrip.js" <<'JS'
;(async () => {
  try { await skenario(); await lapor("selesai.txt", "OK"); }
  catch (e) { try { await potret("gagal"); } catch (_) {} await lapor("selesai.txt", "GAGAL: " + e.message); }
})();
JS
CATAT_DATA_DIR="$DATA" CATAT_UJI_JS="$OUT/skrip.js" CATAT_UJI_OUT="$OUT" \
  "$akar/src-tauri/target/debug/catat" > "$OUT/app.log" 2>&1 &
pid=$!
potret() { # $1 = nama
  if [ -n "${CATAT_POTRET:-}" ]; then $CATAT_POTRET "$OUT/potret-$1.png" >/dev/null 2>&1; fi
}
for _ in $(seq 1 "${CATAT_UJI_BATAS:-600}"); do
  for minta in "$OUT"/potret-*.minta; do
    [ -e "$minta" ] || continue
    n="$(basename "$minta" .minta)"; potret "${n#potret-}"; rm -f "$minta"
  done
  [ -f "$OUT/selesai.txt" ] && break
  kill -0 $pid 2>/dev/null || { echo "aplikasi berhenti sendiri" > "$OUT/selesai.txt"; break; }
  sleep 0.2
done
[ -f "$OUT/selesai.txt" ] || { potret habis-waktu; echo "GAGAL: habis waktu" > "$OUT/selesai.txt"; }
kill $pid 2>/dev/null; wait $pid 2>/dev/null
echo "$nama: $(cat "$OUT/selesai.txt")"
grep -q '^OK' "$OUT/selesai.txt"
