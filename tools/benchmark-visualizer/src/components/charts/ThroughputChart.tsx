import { useMemo } from 'react'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useBenchmark } from '@/context/BenchmarkContext'
import { transformForThroughputChart } from '@/transformers/chartTransformers'
import type { PercentileChartDataPoint } from '@/transformers/chartTransformers'

/**
 * Props for ThroughputChart component
 */
interface ThroughputChartProps {
  framework: string
  fileType: string
  ocrMode: 'no_ocr' | 'with_ocr'
}

/**
 * Color palette for percentile bars
 * Colors that work well in both light and dark themes
 */
const PERCENTILE_COLORS = {
  p50: '#3b82f6',  // Blue - median
  p95: '#8b5cf6',  // Purple - 95th percentile
  p99: '#ef4444',  // Red - 99th percentile
}

/**
 * Tooltip payload structure for recharts
 */
interface TooltipPayload {
  name: string
  value: number
  dataKey?: string
  color?: string
  fill?: string
  payload?: { name: string }
}

/**
 * Tooltip props structure for recharts custom tooltip components
 */
interface TooltipProps {
  active?: boolean
  payload?: TooltipPayload[]
  label?: string
}

/**
 * Custom tooltip component for displaying detailed percentile information
 * Shows full identifier and percentile details
 */
const ThroughputTooltip = ({ active, payload }: TooltipProps) => {
  if (!active || !payload || payload.length === 0) {
    return null
  }

  // Get the full name from the first payload item (all items share the same data point)
  const fullName = (payload[0].payload as any)?.fullName || 'Unknown'

  return (
    <div
      className="rounded-lg border border-border bg-card p-3 shadow-lg"
      data-testid="throughput-tooltip-content"
    >
      <p className="font-semibold text-foreground text-sm">{fullName}</p>
      {payload.map((entry: TooltipPayload, index: number) => (
        <p key={index} style={{ color: entry.color }} className="text-sm">
          <span className="font-medium">{entry.name}:</span> {entry.value?.toFixed(2)} MB/s
        </p>
      ))}
    </div>
  )
}

/**
 * ThroughputChart Component
 *
 * Displays throughput metrics (MB/s) with grouped bars showing p50, p95, and p99 percentiles.
 * Integrates with the BenchmarkContext to fetch data and uses transformForThroughputChart
 * to prepare data for visualization.
 *
 * Features:
 * - Real-time data integration using useBenchmark hook
 * - Percentile-based comparison (p50, p95, p99)
 * - Responsive design with ResponsiveContainer
 * - Light/dark theme support
 * - Loading skeleton while data is fetching
 * - Error alerts for failed data loads
 * - Empty state messaging
 * - Detailed tooltips showing exact values
 *
 * @example
 * ```tsx
 * <ThroughputChart
 *   framework="rust"
 *   fileType="pdf"
 *   ocrMode="no_ocr"
 * />
 * ```
 */
export function ThroughputChart({
  framework,
  fileType,
  ocrMode,
}: ThroughputChartProps) {
  // Get benchmark data from context
  const { data, loading, error } = useBenchmark()

  // Transform raw benchmark data into chart-ready format
  const chartData = useMemo<PercentileChartDataPoint[]>(() => {
    if (!data) return []

    // Transform data using the dedicated transformer
    // This handles percentile extraction and proper filtering
    const transformed = transformForThroughputChart(data, {
      framework,
      fileType,
      ocrMode,
    })

    return transformed
  }, [data, framework, fileType, ocrMode])

  // Loading state
  if (loading) {
    return (
      <Card data-testid="chart-throughput">
        <CardHeader>
          <CardTitle className="text-lg font-semibold">Throughput (MB/s)</CardTitle>
        </CardHeader>
        <CardContent>
          <Skeleton className="h-80 w-full" data-testid="skeleton-throughput-chart" />
        </CardContent>
      </Card>
    )
  }

  // Error state
  if (error) {
    return (
      <Card data-testid="chart-throughput-error">
        <CardHeader>
          <CardTitle className="text-lg font-semibold">Throughput (MB/s)</CardTitle>
        </CardHeader>
        <CardContent>
          <Alert variant="destructive" data-testid="error-throughput-chart">
            <AlertDescription>
              Error loading throughput data: {error.message}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    )
  }

  // Empty state
  if (chartData.length === 0) {
    return (
      <Card data-testid="chart-throughput-empty">
        <CardHeader>
          <CardTitle className="text-lg font-semibold">Throughput (MB/s)</CardTitle>
        </CardHeader>
        <CardContent>
          <div
            className="flex h-80 items-center justify-center text-muted-foreground"
            data-testid="empty-throughput-chart"
          >
            No throughput data available for the selected filters
          </div>
        </CardContent>
      </Card>
    )
  }

  // Main chart render
  return (
    <Card data-testid="chart-throughput">
      <CardHeader>
        <CardTitle className="text-lg font-semibold">Throughput (MB/s)</CardTitle>
        <p className="mt-2 text-sm text-muted-foreground">
          Median (p50), 95th percentile (p95), and 99th percentile (p99) throughput values
        </p>
        <p className="mt-1 text-sm font-medium text-foreground">
          Higher is better
        </p>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={400}>
          <BarChart
            data={chartData}
            margin={{ top: 20, right: 30, left: 0, bottom: 80 }}
            data-testid="throughput-barchart"
          >
            <XAxis
              dataKey="name"
              angle={-45}
              textAnchor="end"
              height={100}
              interval={0}
              tick={{ fontSize: 12 }}
              className="text-muted-foreground"
            />
            <YAxis
              label={{
                value: 'Throughput (MB/s)',
                angle: -90,
                position: 'insideLeft',
                offset: 10,
              }}
              className="text-muted-foreground"
            />
            <Tooltip content={<ThroughputTooltip />} />
            <Legend
              wrapperStyle={{ paddingTop: '20px' }}
              className="text-sm text-muted-foreground"
            />

            {/* P50 Bar - Blue (Median/50th percentile) */}
            <Bar
              dataKey="p50"
              name="P50 (Median)"
              fill={PERCENTILE_COLORS.p50}
              data-testid="bar-throughput-p50"
              radius={[4, 4, 0, 0]}
            />

            {/* P95 Bar - Purple (95th percentile) */}
            <Bar
              dataKey="p95"
              name="P95 (95th Percentile)"
              fill={PERCENTILE_COLORS.p95}
              data-testid="bar-throughput-p95"
              radius={[4, 4, 0, 0]}
            />

            {/* P99 Bar - Red (99th percentile) */}
            <Bar
              dataKey="p99"
              name="P99 (99th Percentile)"
              fill={PERCENTILE_COLORS.p99}
              data-testid="bar-throughput-p99"
              radius={[4, 4, 0, 0]}
            />
          </BarChart>
        </ResponsiveContainer>

        {/* Legend explanation */}
        <div className="mt-6 grid grid-cols-3 gap-4 rounded-lg bg-muted/50 p-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <div
                className="h-3 w-3 rounded"
                style={{ backgroundColor: PERCENTILE_COLORS.p50 }}
              />
              <span className="text-sm font-medium">P50 (Median)</span>
            </div>
            <p className="text-xs text-muted-foreground">
              Typical throughput for 50% of requests
            </p>
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <div
                className="h-3 w-3 rounded"
                style={{ backgroundColor: PERCENTILE_COLORS.p95 }}
              />
              <span className="text-sm font-medium">P95 (95th %ile)</span>
            </div>
            <p className="text-xs text-muted-foreground">
              Throughput for 95% of requests
            </p>
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <div
                className="h-3 w-3 rounded"
                style={{ backgroundColor: PERCENTILE_COLORS.p99 }}
              />
              <span className="text-sm font-medium">P99 (99th %ile)</span>
            </div>
            <p className="text-xs text-muted-foreground">
              Throughput for slowest 1% of requests
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
