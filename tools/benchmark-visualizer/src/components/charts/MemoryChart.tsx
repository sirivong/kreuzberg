import { useMemo } from 'react'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  Legend,
  ResponsiveContainer,
  CartesianGrid,
} from 'recharts'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useBenchmark } from '@/context/BenchmarkContext'
import { transformForMemoryChart } from '@/transformers/chartTransformers'
import type { PercentileChartDataPoint } from '@/transformers/chartTransformers'

interface MemoryChartProps {
  framework: string
  fileType: string
  ocrMode: 'no_ocr' | 'with_ocr'
}

/**
 * Custom tooltip component for memory chart
 * Displays exact memory values with MB unit for each percentile
 */
interface TooltipPayload {
  name: string
  value: number
  dataKey?: string
  fill?: string
  payload?: { fullName?: string }
}

interface CustomTooltipProps {
  active?: boolean
  payload?: TooltipPayload[]
  label?: string
}

const CustomTooltip = ({ active, payload }: CustomTooltipProps) => {
  if (!active || !payload || payload.length === 0) {
    return null
  }

  // Get the full name from the first payload item (all items share the same data point)
  const fullName = (payload[0].payload as any)?.fullName || 'Unknown'

  return (
    <div className="bg-slate-900 border border-slate-700 rounded-md p-3 shadow-lg">
      <p className="text-sm font-semibold text-white">{fullName}</p>
      {payload.map((entry: TooltipPayload, index: number) => (
        <p key={index} style={{ color: entry.fill || '#ffffff' }} className="text-sm">
          {entry.name}: {Number(entry.value).toFixed(2)} MB
        </p>
      ))}
    </div>
  )
}

/**
 * Color palette for percentiles that works in both light and dark themes
 * p50: Blue (baseline), p95: Orange (warning), p99: Red (critical)
 */
const PERCENTILE_COLORS = {
  p50: '#3b82f6', // Blue
  p95: '#f97316', // Orange
  p99: '#ef4444', // Red
}

export function MemoryChart({
  framework,
  fileType,
  ocrMode,
}: MemoryChartProps) {
  const { data, loading, error } = useBenchmark()

  // Transform data for memory chart with optional filters
  const chartData = useMemo<PercentileChartDataPoint[]>(() => {
    if (!data) return []

    return transformForMemoryChart(data, {
      framework,
      fileType,
      ocrMode,
    })
  }, [data, framework, fileType, ocrMode])

  if (loading) {
    return (
      <Card data-testid="chart-memory">
        <CardHeader>
          <CardTitle>Memory Usage (MB)</CardTitle>
          <p className="text-sm text-muted-foreground mt-2">
            Shows p50, p95, and p99 percentiles for memory consumption
          </p>
        </CardHeader>
        <CardContent>
          <Skeleton className="h-80 w-full" data-testid="skeleton-memory-chart" />
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card data-testid="chart-memory-error">
        <CardHeader>
          <CardTitle>Memory Usage (MB)</CardTitle>
          <p className="text-sm text-muted-foreground mt-2">
            Shows p50, p95, and p99 percentiles for memory consumption
          </p>
        </CardHeader>
        <CardContent>
          <Alert variant="destructive" data-testid="error-memory-chart">
            <AlertDescription>
              Error loading memory data: {error instanceof Error ? error.message : 'Unknown error'}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    )
  }

  if (chartData.length === 0) {
    return (
      <Card data-testid="chart-memory-empty">
        <CardHeader>
          <CardTitle>Memory Usage (MB)</CardTitle>
          <p className="text-sm text-muted-foreground mt-2">
            Shows p50, p95, and p99 percentiles for memory consumption
          </p>
        </CardHeader>
        <CardContent>
          <div
            className="flex flex-col items-center justify-center h-80 text-muted-foreground"
            data-testid="empty-memory-chart"
          >
            <p className="text-lg mb-2">No data available</p>
            <p className="text-sm">Try adjusting your filters to view memory usage data</p>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card data-testid="chart-memory">
      <CardHeader>
        <CardTitle>Memory Usage (MB)</CardTitle>
        <p className="text-sm text-muted-foreground mt-2">
          Shows p50 (median), p95, and p99 percentiles for memory consumption
        </p>
        <p className="mt-1 text-sm font-medium text-foreground">
          Lower is better
        </p>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={400} data-testid="memory-responsive-container">
          <BarChart
            data={chartData}
            margin={{ top: 20, right: 30, left: 0, bottom: 60 }}
            data-testid="memory-barchart"
          >
            <CartesianGrid strokeDasharray="3 3" stroke="currentColor" opacity={0.1} />
            <XAxis
              dataKey="name"
              angle={-45}
              textAnchor="end"
              height={100}
              interval={0}
              tick={{ fontSize: 12 }}
              data-testid="memory-xaxis"
            />
            <YAxis
              label={{ value: 'Memory (MB)', angle: -90, position: 'insideLeft' }}
              data-testid="memory-yaxis"
            />
            <Tooltip
              content={<CustomTooltip />}
              data-testid="memory-tooltip"
              cursor={{ fill: 'rgba(0, 0, 0, 0.05)' }}
            />
            <Legend
              wrapperStyle={{ paddingTop: '20px' }}
              data-testid="memory-legend"
              verticalAlign="top"
              height={36}
            />
            <Bar
              dataKey="p50"
              fill={PERCENTILE_COLORS.p50}
              name="p50 (Median)"
              data-testid="bar-memory-p50"
              radius={[8, 8, 0, 0]}
            />
            <Bar
              dataKey="p95"
              fill={PERCENTILE_COLORS.p95}
              name="p95 (95th %ile)"
              data-testid="bar-memory-p95"
              radius={[8, 8, 0, 0]}
            />
            <Bar
              dataKey="p99"
              fill={PERCENTILE_COLORS.p99}
              name="p99 (99th %ile)"
              data-testid="bar-memory-p99"
              radius={[8, 8, 0, 0]}
            />
          </BarChart>
        </ResponsiveContainer>
        <div className="mt-6 grid grid-cols-3 gap-4 text-sm">
          <div className="flex items-center gap-2">
            <div
              className="w-4 h-4 rounded"
              style={{ backgroundColor: PERCENTILE_COLORS.p50 }}
            />
            <span className="text-muted-foreground">p50: Typical memory usage</span>
          </div>
          <div className="flex items-center gap-2">
            <div
              className="w-4 h-4 rounded"
              style={{ backgroundColor: PERCENTILE_COLORS.p95 }}
            />
            <span className="text-muted-foreground">p95: High memory usage</span>
          </div>
          <div className="flex items-center gap-2">
            <div
              className="w-4 h-4 rounded"
              style={{ backgroundColor: PERCENTILE_COLORS.p99 }}
            />
            <span className="text-muted-foreground">p99: Peak memory usage</span>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
