import { useMemo } from 'react'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from 'recharts'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useBenchmark } from '@/context/BenchmarkContext'
import { formatFramework } from '@/transformers/chartTransformers'

/**
 * Tooltip payload structure for recharts
 */
interface TooltipPayload {
  name: string
  value: number
  dataKey?: string
  color?: string
  fill?: string
  payload?: {
    name: string
    description: string
    sizeInMb: number
  }
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
 * Custom tooltip to show exact disk size and description
 */
const CustomTooltip = ({ active, payload }: TooltipProps) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload
    return (
      <div className="bg-slate-900 text-white p-3 rounded-lg border border-slate-700 shadow-lg">
        <p className="font-semibold text-sm">{data?.name}</p>
        {data?.description && (
          <p className="text-xs text-slate-300 mt-1">{data.description}</p>
        )}
        <p className="text-sm mt-2">
          Size: <span className="font-semibold">{data?.sizeInMb.toFixed(2)} MB</span>
        </p>
      </div>
    )
  }
  return null
}

/**
 * Color for the disk size bar
 */
const BAR_COLOR = '#3b82f6' // Blue

interface ChartDataPoint {
  name: string
  sizeInMb: number
  description: string
}

export function DiskSizeChart() {
  const { data, loading, error } = useBenchmark()

  const chartData = useMemo<ChartDataPoint[]>(() => {
    if (!data?.disk_sizes) return []

    // Transform disk_sizes data to chart format
    const transformed = Object.entries(data.disk_sizes).map(([framework, info]) => ({
      name: formatFramework(framework),
      sizeInMb: info.size_bytes / (1024 * 1024),
      description: info.description,
    }))

    // Sort by size (smallest to largest)
    return transformed.sort((a, b) => a.sizeInMb - b.sizeInMb)
  }, [data?.disk_sizes])

  if (loading) {
    return (
      <Card data-testid="chart-disk-size">
        <CardHeader>
          <CardTitle>Disk Size Comparison</CardTitle>
        </CardHeader>
        <CardContent>
          <Skeleton className="h-80 w-full" data-testid="skeleton-disk-size-chart" />
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card data-testid="chart-disk-size-error">
        <CardHeader>
          <CardTitle>Disk Size Comparison</CardTitle>
        </CardHeader>
        <CardContent>
          <Alert variant="destructive" data-testid="error-disk-size-chart">
            <AlertDescription>
              Error loading disk size data: {error instanceof Error ? error.message : 'Unknown error'}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    )
  }

  if (chartData.length === 0) {
    return (
      <Card data-testid="chart-disk-size-empty">
        <CardHeader>
          <CardTitle>Disk Size Comparison</CardTitle>
        </CardHeader>
        <CardContent>
          <div
            className="flex items-center justify-center h-80 text-muted-foreground"
            data-testid="empty-disk-size-chart"
          >
            No disk size data available
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card data-testid="chart-disk-size">
      <CardHeader>
        <CardTitle>Disk Size Comparison</CardTitle>
        <p className="text-sm text-muted-foreground mt-2">
          Framework disk sizes sorted from smallest to largest.
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
            data-testid="disk-size-barchart"
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
              label={{ value: 'Size (MB)', angle: -90, position: 'insideLeft' }}
              tick={{ fontSize: 12 }}
            />
            <Tooltip
              content={<CustomTooltip />}
              data-testid="disk-size-tooltip"
              cursor={{ fill: 'rgba(59, 130, 246, 0.1)' }}
            />
            <Bar
              dataKey="sizeInMb"
              fill={BAR_COLOR}
              data-testid="bar-disk-size"
              radius={[8, 8, 0, 0]}
              name="Disk Size (MB)"
            />
          </BarChart>
        </ResponsiveContainer>
      </CardContent>
    </Card>
  )
}
