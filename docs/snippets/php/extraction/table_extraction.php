```php
<?php

declare(strict_types=1);

/**
 * Table Extraction and Processing
 *
 * Extract tables from PDFs and other documents, process them,
 * and export to various formats (CSV, JSON, HTML).
 */

require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;
use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\OcrConfig;
use Kreuzberg\Config\TesseractConfig;

// Basic table extraction
$config = new ExtractionConfig(extractTables: true);
$kreuzberg = new Kreuzberg($config);
$result = $kreuzberg->extractFile('financial_report.pdf');

echo "Table Extraction:\n";
echo str_repeat('=', 60) . "\n";
echo "Tables found: " . count($result->tables) . "\n\n";

// Display tables in different formats
foreach ($result->tables as $index => $table) {
    echo "Table " . ($index + 1) . " (Page {$table->pageNumber}):\n";
    echo str_repeat('-', 60) . "\n";

    // Format 1: Markdown
    echo "Markdown:\n";
    echo $table->markdown . "\n\n";

    // Format 2: Raw array
    echo "Array format:\n";
    echo "Rows: " . count($table->cells) . "\n";
    echo "Columns: " . (count($table->cells[0] ?? []) ?? 0) . "\n\n";

    // Format 3: HTML
    echo "HTML:\n";
    echo "<table>\n";
    foreach ($table->cells as $rowIndex => $row) {
        $tag = $rowIndex === 0 ? 'th' : 'td';
        echo "  <tr>\n";
        foreach ($row as $cell) {
            echo "    <$tag>" . htmlspecialchars($cell) . "</$tag>\n";
        }
        echo "  </tr>\n";
    }
    echo "</table>\n\n";
}

// Export tables to CSV files
foreach ($result->tables as $index => $table) {
    $filename = "table_" . ($index + 1) . "_page_" . $table->pageNumber . ".csv";
    $fp = fopen($filename, 'w');

    foreach ($table->cells as $row) {
        fputcsv($fp, $row);
    }

    fclose($fp);
    echo "Exported to: $filename\n";
}
echo "\n";

// Extract tables with OCR (for scanned documents)
$ocrConfig = new ExtractionConfig(
    extractTables: true,
    ocr: new OcrConfig(
        backend: 'tesseract',
        language: 'eng',
        tesseractConfig: new TesseractConfig(
            enableTableDetection: true,
            psm: 6
        )
    )
);

$kreuzberg = new Kreuzberg($ocrConfig);
$result = $kreuzberg->extractFile('scanned_table.pdf');

echo "OCR Table Extraction:\n";
echo str_repeat('=', 60) . "\n";
echo "Tables with OCR: " . count($result->tables) . "\n\n";

// Process table data
function processTable(array $cells): array
{
    $processed = [];

    // Skip header row
    $headers = array_shift($cells);

    foreach ($cells as $row) {
        $rowData = [];
        foreach ($headers as $index => $header) {
            $rowData[$header] = $row[$index] ?? '';
        }
        $processed[] = $rowData;
    }

    return $processed;
}

foreach ($result->tables as $table) {
    $structured = processTable($table->cells);

    echo "Structured table data:\n";
    echo json_encode($structured, JSON_PRETTY_PRINT) . "\n\n";
}

// Filter tables by content
function findTablesWithKeyword(array $tables, string $keyword): array
{
    $matching = [];

    foreach ($tables as $table) {
        foreach ($table->cells as $row) {
            foreach ($row as $cell) {
                if (stripos($cell, $keyword) !== false) {
                    $matching[] = $table;
                    break 2;
                }
            }
        }
    }

    return $matching;
}

$salesTables = findTablesWithKeyword($result->tables, 'sales');
echo "Tables containing 'sales': " . count($salesTables) . "\n";

// Convert table to associative array with headers
function tableToAssociativeArray(\Kreuzberg\Types\Table $table): array
{
    $cells = $table->cells;
    if (empty($cells)) {
        return [];
    }

    $headers = array_shift($cells);
    $result = [];

    foreach ($cells as $row) {
        $rowData = [];
        foreach ($headers as $index => $header) {
            $rowData[$header] = $row[$index] ?? null;
        }
        $result[] = $rowData;
    }

    return $result;
}

// Example: Extract financial data
$result = $kreuzberg->extractFile('quarterly_report.pdf');

foreach ($result->tables as $index => $table) {
    $data = tableToAssociativeArray($table);

    echo "\nTable " . ($index + 1) . " data:\n";

    // Calculate totals if numeric columns
    $totals = [];
    foreach ($data as $row) {
        foreach ($row as $key => $value) {
            if (is_numeric($value)) {
                if (!isset($totals[$key])) {
                    $totals[$key] = 0;
                }
                $totals[$key] += floatval($value);
            }
        }
    }

    if (!empty($totals)) {
        echo "Column totals:\n";
        foreach ($totals as $column => $total) {
            echo "  $column: " . number_format($total, 2) . "\n";
        }
    }
}

// Export all tables to JSON
$allTablesJson = array_map(function ($table) {
    return [
        'page' => $table->pageNumber,
        'rows' => count($table->cells),
        'columns' => count($table->cells[0] ?? []),
        'data' => tableToAssociativeArray($table),
        'markdown' => $table->markdown,
    ];
}, $result->tables);

file_put_contents('tables.json', json_encode($allTablesJson, JSON_PRETTY_PRINT));
echo "\nAll tables exported to: tables.json\n";

// Merge tables from multiple pages
function mergeTables(array $tables): array
{
    if (empty($tables)) {
        return [];
    }

    $merged = [];
    $headers = $tables[0]->cells[0] ?? [];

    foreach ($tables as $table) {
        $cells = $table->cells;
        array_shift($cells); // Remove headers from subsequent tables

        foreach ($cells as $row) {
            $merged[] = $row;
        }
    }

    return ['headers' => $headers, 'data' => $merged];
}

// Example: Merge tables from multi-page report
$reportTables = findTablesWithKeyword($result->tables, 'Quarter');
if (!empty($reportTables)) {
    $merged = mergeTables($reportTables);
    echo "\nMerged " . count($reportTables) . " tables\n";
    echo "Total rows: " . count($merged['data']) . "\n";
}
```
