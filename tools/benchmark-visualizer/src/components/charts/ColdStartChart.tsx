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
import { transformForColdStartChart } from '@/transformers/chartTransformers'
import type { AggregatedBenchmarkData } from '@/types/benchmark'

interface ColdStartChartProps {
  data?: AggregatedBenchmarkData | null
  loading?: boolean
  error?: Error | null
  framework: string
}

/**
 * Color palette for percentiles that works in both light and dark themes
 * Using colors with good contrast and visual distinction
 */
const PERCENTILE_COLORS = {
  p50: '#3b82f6', // Blue - median/typical case
  p95: '#f59e0b', // Amber - 95th percentile
  p99: '#ef4444', // Red - worst case (99th percentile)
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
 * Custom tooltip to show full identifier and percentile information
 */
const CustomTooltip = ({ active, payload }: TooltipProps) => {
  if (active && payload && payload.length) {
    // Get the full name from the first payload item (all items share the same data point)
    const fullName = (payload[0].payload as any)?.fullName || 'Unknown'

    return (
      <div className="bg-slate-900 text-white p-3 rounded-lg border border-slate-700 shadow-lg">
        <p className="font-semibold text-sm">{fullName}</p>
        {payload.map((entry: TooltipPayload, index: number) => (
          <p key={`item-${index}`} style={{ color: entry.color }} className="text-sm">
            {entry.name}: {entry.value.toFixed(2)} ms
          </p>
        ))}
      </div>
    )
  }
  return null
}

export function ColdStartChart({
  data: externalData,
  loading: externalLoading,
  error: externalError,
  framework,
}: ColdStartChartProps) {
  // Use context data if not provided externally
  const contextData = useBenchmark()
  const data = externalData ?? contextData.data
  const loading = externalLoading ?? contextData.loading
  const error = externalError ?? contextData.error

  const chartData = useMemo(() => {
    if (!data) return []

    // Transform data using the dedicated transformer
    return transformForColdStartChart(data, { framework })
  }, [data, framework])

  if (loading) {
    return (
      <Card data-testid="chart-cold-start">
        <CardHeader>
          <CardTitle>Cold Start Time (ms)</CardTitle>
        </CardHeader>
        <CardContent>
          <Skeleton className="h-80 w-full" data-testid="skeleton-cold-start-chart" />
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card data-testid="chart-cold-start-error">
        <CardHeader>
          <CardTitle>Cold Start Time (ms)</CardTitle>
        </CardHeader>
        <CardContent>
          <Alert variant="destructive" data-testid="error-cold-start-chart">
            <AlertDescription>Error loading chart: {error.message}</AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    )
  }

  if (chartData.length === 0) {
    return (
      <Card data-testid="chart-cold-start-empty">
        <CardHeader>
          <CardTitle>Cold Start Time (ms)</CardTitle>
        </CardHeader>
        <CardContent>
          <div
            className="flex items-center justify-center h-80 text-muted-foreground"
            data-testid="empty-cold-start-chart"
          >
            No cold start data available for the selected filters
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card data-testid="chart-cold-start">
      <CardHeader>
        <CardTitle>Cold Start Time (ms)</CardTitle>
        <p className="text-sm text-muted-foreground mt-2">
          Comparing p50 (median), p95, and p99 (worst case) cold start times across frameworks
        </p>
        <p className="mt-1 text-sm font-medium text-foreground">
          Lower is better
        </p>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={400}>
          <BarChart
            data={chartData}
            margin={{ top: 20, right: 30, left: 0, bottom: 80 }}
            data-testid="cold-start-barchart"
          >
            <XAxis
              dataKey="name"
              angle={-45}
              textAnchor="end"
              height={120}
              interval={0}
              tick={{ fontSize: 12 }}
            />
            <YAxis
              label={{ value: 'Cold Start Time (ms)', angle: -90, position: 'insideLeft' }}
              tick={{ fontSize: 12 }}
            />
            <Tooltip
              content={<CustomTooltip />}
              data-testid="cold-start-tooltip"
              cursor={{ fill: 'rgba(59, 130, 246, 0.1)' }}
            />
            <Legend
              wrapperStyle={{ paddingTop: '20px' }}
              data-testid="cold-start-legend"
              formatter={value => {
                const labels: Record<string, string> = {
                  p50: 'p50 (Median)',
                  p95: 'p95 (95th Percentile)',
                  p99: 'p99 (99th Percentile)',
                }
                return labels[value] || value
              }}
            />
            <Bar
              dataKey="p50"
              fill={PERCENTILE_COLORS.p50}
              data-testid="bar-cold-start-p50"
              name="p50"
            />
            <Bar
              dataKey="p95"
              fill={PERCENTILE_COLORS.p95}
              data-testid="bar-cold-start-p95"
              name="p95"
            />
            <Bar
              dataKey="p99"
              fill={PERCENTILE_COLORS.p99}
              data-testid="bar-cold-start-p99"
              name="p99"
            />
          </BarChart>
        </ResponsiveContainer>
        <div className="mt-6 grid grid-cols-3 gap-4 text-sm">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: PERCENTILE_COLORS.p50 }} />
            <span className="text-muted-foreground">p50: Median response time</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: PERCENTILE_COLORS.p95 }} />
            <span className="text-muted-foreground">p95: 95% of requests faster</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: PERCENTILE_COLORS.p99 }} />
            <span className="text-muted-foreground">p99: Worst case scenario</span>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
