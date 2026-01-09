import { useState, useMemo } from 'react'
import { useBenchmark } from '@/context/BenchmarkContext'
import { Skeleton } from '@/components/ui/skeleton'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { FrameworkFilter } from '@/components/filters/FrameworkFilter'
import { FileTypeFilter } from '@/components/filters/FileTypeFilter'
import { OCRModeFilter } from '@/components/filters/OCRModeFilter'
import { ThroughputChart } from '@/components/charts/ThroughputChart'
import { MemoryChart } from '@/components/charts/MemoryChart'
import { DurationChart } from '@/components/charts/DurationChart'
import { ColdStartChart } from '@/components/charts/ColdStartChart'
import { DiskSizeChart } from '@/components/charts/DiskSizeChart'
import { getFrameworkCapabilities, filterFrameworksByFileType } from '@/utils/frameworkCapabilities'
import type { AggregatedBenchmarkData } from '@/types/benchmark'

export function PerformanceChartsPage() {
  const { data, loading, error } = useBenchmark()
  const [selectedFrameworks, setSelectedFrameworks] = useState<string[]>([])
  const [selectedFileTypes, setSelectedFileTypes] = useState<string[]>(['pdf']) // Default to PDF
  const [ocrMode, setOcrMode] = useState<'' | 'no_ocr' | 'with_ocr'>('no_ocr') // Default to no OCR

  // Calculate framework capabilities and filtered data
  const { filteredData, frameworkSupport } = useMemo(() => {
    if (!data) {
      return { filteredData: null, frameworkSupport: { supporting: 0, total: 0 } }
    }

    const selectedFileType = selectedFileTypes[0]
    if (!selectedFileType) {
      return { filteredData: data, frameworkSupport: { supporting: 0, total: 0 } }
    }

    // Get framework capabilities
    const capabilities = getFrameworkCapabilities(data)

    // Get all framework keys
    const allFrameworkKeys = Object.keys(data.by_framework_mode)

    // Filter frameworks that support the selected file type
    const supportedFrameworkKeys = filterFrameworksByFileType(
      allFrameworkKeys,
      selectedFileType,
      capabilities
    )

    // Create filtered data with only supported frameworks
    const filteredByFrameworkMode: Record<string, typeof data.by_framework_mode[string]> = {}
    supportedFrameworkKeys.forEach(key => {
      filteredByFrameworkMode[key] = data.by_framework_mode[key]
    })

    const filteredBenchmarkData: AggregatedBenchmarkData = {
      ...data,
      by_framework_mode: filteredByFrameworkMode,
    }

    return {
      filteredData: filteredBenchmarkData,
      frameworkSupport: {
        supporting: supportedFrameworkKeys.length,
        total: allFrameworkKeys.length,
      },
    }
  }, [data, selectedFileTypes])

  if (loading) {
    return (
      <div className="container mx-auto p-4">
        <Skeleton className="h-12 w-64 mb-6" data-testid="skeleton-charts" />
        <Skeleton className="h-96" />
      </div>
    )
  }

  if (error) {
    return (
      <div className="container mx-auto p-4">
        <Alert variant="destructive" data-testid="error-message">
          <AlertDescription>Error: {error.message}</AlertDescription>
        </Alert>
      </div>
    )
  }

  if (!data || !filteredData) {
    return null
  }

  // Get the first selected framework and file type for charts
  const selectedFramework = selectedFrameworks[0]
  const selectedFileType = selectedFileTypes[0]

  // Check if minimum required filters are selected
  const hasRequiredFilters = selectedFileType && ocrMode

  // Check if any frameworks were filtered out
  const hasFilteredFrameworks = frameworkSupport.supporting < frameworkSupport.total

  return (
    <div data-testid="page-charts" className="container mx-auto p-4">
      <h1 className="text-4xl font-bold mb-6">Performance Charts</h1>

      <div className="mb-6 flex gap-4">
        <FrameworkFilter
          selectedFrameworks={selectedFrameworks}
          onFrameworksChange={setSelectedFrameworks}
          data-testid="filters-framework"
        />
        <FileTypeFilter
          selectedFileTypes={selectedFileTypes}
          onFileTypesChange={setSelectedFileTypes}
          data-testid="filters-file-type"
        />
        <OCRModeFilter
          selectedOCRMode={ocrMode}
          onOCRModeChange={setOcrMode}
          data-testid="filter-ocr"
        />
      </div>

      {!hasRequiredFilters ? (
        <Alert data-testid="validation-message">
          <AlertDescription>
            Select a file type and OCR mode to view charts
          </AlertDescription>
        </Alert>
      ) : (
        <>
          {hasFilteredFrameworks && (
            <Alert className="mb-6" data-testid="framework-filter-indicator">
              <AlertDescription>
                Showing {frameworkSupport.supporting} of {frameworkSupport.total} frameworks that support {selectedFileType.toUpperCase()}
              </AlertDescription>
            </Alert>
          )}
          <div className="space-y-6">
            <ThroughputChart
              framework={selectedFramework}
              fileType={selectedFileType}
              ocrMode={ocrMode}
            />

            <MemoryChart
              framework={selectedFramework}
              fileType={selectedFileType}
              ocrMode={ocrMode}
            />

            <DurationChart
              fileType={selectedFileType}
              ocrMode={ocrMode}
            />

            <ColdStartChart
              framework={selectedFramework}
            />

            <DiskSizeChart />
          </div>
        </>
      )}
    </div>
  )
}
