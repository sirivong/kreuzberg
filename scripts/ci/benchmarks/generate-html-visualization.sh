#!/usr/bin/env bash
set -euo pipefail

HARNESS="./target/release/benchmark-harness"
RESULTS_ROOT="benchmark-results"
OUTPUT_ROOT="benchmark-output"

if [ ! -d "${RESULTS_ROOT}" ]; then
	echo "::error::Missing benchmark results directory: ${RESULTS_ROOT}" >&2
	exit 1
fi

mapfile -t result_dirs < <(find "${RESULTS_ROOT}" -mindepth 1 -maxdepth 1 -type d | sort)

if [ "${#result_dirs[@]}" -eq 0 ]; then
	echo "::error::No benchmark result directories found under ${RESULTS_ROOT}" >&2
	exit 1
fi

single_dirs=()
batch_dirs=()

for dir in "${result_dirs[@]}"; do
	name="$(basename "$dir")"
	if [[ "$name" == *"single-file" ]]; then
		single_dirs+=("$dir")
	elif [[ "$name" == *"batch" ]]; then
		batch_dirs+=("$dir")
	fi
done

mkdir -p "${OUTPUT_ROOT}"

if [ "${#single_dirs[@]}" -gt 0 ]; then
	single_inputs="$(
		IFS=,
		echo "${single_dirs[*]}"
	)"
	"${HARNESS}" visualize \
		--inputs "${single_inputs}" \
		--output "${OUTPUT_ROOT}/single-file" \
		--format both
fi

if [ "${#batch_dirs[@]}" -gt 0 ]; then
	batch_inputs="$(
		IFS=,
		echo "${batch_dirs[*]}"
	)"
	"${HARNESS}" visualize \
		--inputs "${batch_inputs}" \
		--output "${OUTPUT_ROOT}/batch" \
		--format both
fi

{
	echo "<!doctype html>"
	echo "<html lang=\"en\">"
	echo "<head>"
	echo "  <meta charset=\"utf-8\">"
	echo "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
	echo "  <title>Kreuzberg Benchmarks</title>"
	echo "  <style>"
	echo "    body { font-family: system-ui, -apple-system, sans-serif; margin: 0; padding: 24px; background: #f7f7f5; color: #1f2933; }"
	echo "    h1 { margin: 0 0 12px; }"
	echo "    .tabs { display: flex; gap: 12px; margin-bottom: 16px; flex-wrap: wrap; }"
	echo "    .tab { border: 1px solid #d1d5db; padding: 8px 16px; border-radius: 999px; background: white; cursor: pointer; }"
	echo "    .tab.active { background: #1f2933; color: white; border-color: #1f2933; }"
	echo "    .panel { display: none; }"
	echo "    .panel.active { display: block; }"
	echo "    iframe { width: 100%; height: 2000px; border: 1px solid #d1d5db; border-radius: 12px; background: white; }"
	echo "    .note { color: #52616b; margin-bottom: 20px; }"
	echo "  </style>"
	echo "</head>"
	echo "<body>"
	echo "  <h1>Kreuzberg Benchmark Dashboard</h1>"
	echo "  <p class=\"note\">Select a benchmark mode to view comparative results.</p>"
	echo "  <div class=\"tabs\">"
	if [ "${#single_dirs[@]}" -gt 0 ]; then
		echo "    <button class=\"tab active\" data-target=\"single\">Single-file</button>"
	fi
	if [ "${#batch_dirs[@]}" -gt 0 ]; then
		if [ "${#single_dirs[@]}" -gt 0 ]; then
			echo "    <button class=\"tab\" data-target=\"batch\">Batch</button>"
		else
			echo "    <button class=\"tab active\" data-target=\"batch\">Batch</button>"
		fi
	fi
	echo "  </div>"
	if [ "${#single_dirs[@]}" -gt 0 ]; then
		echo "  <div class=\"panel active\" id=\"panel-single\">"
		echo "    <iframe src=\"single-file/index.html\" title=\"Single-file benchmarks\"></iframe>"
		echo "  </div>"
	fi
	if [ "${#batch_dirs[@]}" -gt 0 ]; then
		panel_class=""
		if [ "${#single_dirs[@]}" -eq 0 ]; then
			panel_class=" active"
		fi
		echo "  <div class=\"panel${panel_class}\" id=\"panel-batch\">"
		echo "    <iframe src=\"batch/index.html\" title=\"Batch benchmarks\"></iframe>"
		echo "  </div>"
	fi
	echo "  <script>"
	echo "    const tabs = document.querySelectorAll('.tab');"
	echo "    const panels = document.querySelectorAll('.panel');"
	echo "    tabs.forEach((tab) => {"
	echo "      tab.addEventListener('click', () => {"
	echo "        tabs.forEach((t) => t.classList.remove('active'));"
	echo "        panels.forEach((p) => p.classList.remove('active'));"
	echo "        tab.classList.add('active');"
	echo "        const target = tab.dataset.target;"
	echo "        const panel = document.getElementById('panel-' + target);"
	echo "        if (panel) { panel.classList.add('active'); }"
	echo "      });"
	echo "    });"
	echo "  </script>"
	echo "</body>"
	echo "</html>"
} >"${OUTPUT_ROOT}/index.html"
