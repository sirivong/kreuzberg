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
import { transformForDurationChart } from '@/transformers/chartTransformers'

interface DurationChartProps {
  fileType: string
  ocrMode: 'no_ocr' | 'with_ocr'
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
 * Tooltip content renderer for duration chart
 * Shows full identifier and exact values with proper formatting
 */
function DurationTooltip({
  active,
  payload,
}: TooltipProps) {
  if (active && payload && payload.length) {
    // Get the full name from the first payload item (all items share the same data point)
    const fullName = (payload[0].payload as any)?.fullName || 'Unknown'

    return (
      <div className="bg-slate-900 text-white p-3 rounded border border-slate-700">
        <p className="font-semibold text-sm">{fullName}</p>
        {payload.map((entry, index) => (
          <p key={index} style={{ color: entry.fill }} className="text-sm">
            {entry.dataKey}: {entry.value.toFixed(2)} ms
          </p>
        ))}
      </div>
    )
  }
  return null
}

export function DurationChart({ fileType, ocrMode }: DurationChartProps) {
  const { data, loading, error } = useBenchmark()

  const chartData = useMemo(() => {
    if (!data) return []

    return transformForDurationChart(data, {
      fileType,
      ocrMode,
    })
  }, [data, fileType, ocrMode])

  if (loading) {
    return (
      <Card data-testid="chart-duration">
        <CardHeader>
          <CardTitle>Duration (ms) - p50, p95, p99 Percentiles</CardTitle>
        </CardHeader>
        <CardContent>
          <Skeleton className="h-96 w-full" data-testid="skeleton-duration-chart" />
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card data-testid="chart-duration-error">
        <CardHeader>
          <CardTitle>Duration (ms) - p50, p95, p99 Percentiles</CardTitle>
        </CardHeader>
        <CardContent>
          <Alert variant="destructive" data-testid="error-duration-chart">
            <AlertDescription>Error loading duration chart: {error.message}</AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    )
  }

  if (chartData.length === 0) {
    return (
      <Card data-testid="chart-duration-empty">
        <CardHeader>
          <CardTitle>Duration (ms) - p50, p95, p99 Percentiles</CardTitle>
        </CardHeader>
        <CardContent>
          <div
            className="flex flex-col items-center justify-center h-96 text-muted-foreground gap-2"
            data-testid="empty-duration-chart"
          >
            <p>No duration data available for the selected filters</p>
            <p className="text-sm">Try adjusting your file type or OCR mode selection</p>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card data-testid="chart-duration">
      <CardHeader>
        <CardTitle>Duration (ms) - p50, p95, p99 Percentiles</CardTitle>
        <p className="text-sm text-muted-foreground mt-2">
          Percentiles show performance distribution across benchmark runs.
        </p>
        <p className="mt-1 text-sm font-medium text-foreground">
          Lower is better
        </p>
      </CardHeader>
      <CardContent>
        <div className="w-full h-96 -ml-4">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart
              data={chartData}
              margin={{ top: 20, right: 30, left: 60, bottom: 80 }}
              data-testid="duration-barchart"
            >
              <XAxis
                dataKey="name"
                angle={-45}
                textAnchor="end"
                height={120}
                interval={0}
                tick={{ fontSize: 12 }}
                className="text-foreground"
              />
              <YAxis
                label={{ value: 'Duration (ms)', angle: -90, position: 'insideLeft', offset: 10 }}
                className="text-foreground"
              />
              <Tooltip
                content={<DurationTooltip />}
                data-testid="duration-tooltip"
                cursor={{ fill: 'rgba(0, 0, 0, 0.05)' }}
              />
              <Legend
                wrapperStyle={{ paddingTop: '20px' }}
                iconType="square"
                formatter={(value) => {
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
                fill="hsl(220, 90%, 56%)"
                name="p50"
                data-testid="bar-duration-p50"
                radius={[4, 4, 0, 0]}
              />
              <Bar
                dataKey="p95"
                fill="hsl(16, 100%, 50%)"
                name="p95"
                data-testid="bar-duration-p95"
                radius={[4, 4, 0, 0]}
              />
              <Bar
                dataKey="p99"
                fill="hsl(0, 84%, 60%)"
                name="p99"
                data-testid="bar-duration-p99"
                radius={[4, 4, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  )
}
