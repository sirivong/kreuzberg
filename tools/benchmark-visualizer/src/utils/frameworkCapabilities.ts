import type { AggregatedBenchmarkData } from '@/types/benchmark'

/**
 * Threshold for determining if a framework supports a file type.
 * If p50 duration is greater than this value (in milliseconds),
 * we consider the framework doesn't support that file type.
 */
const UNSUPPORTED_THRESHOLD_MS = 10000 // 10 seconds

/**
 * Analyzes benchmark data to determine which frameworks support which file types.
 * A framework is considered to support a file type if its p50 duration is below
 * the UNSUPPORTED_THRESHOLD_MS (indicating successful processing rather than timeout).
 *
 * @param data - The aggregated benchmark data
 * @returns A Map where keys are framework-mode identifiers (e.g., "rust-single")
 *          and values are Sets of supported file types (e.g., "pdf", "docx")
 *
 * @example
 * const capabilities = getFrameworkCapabilities(data)
 * const rustSingleSupports = capabilities.get('rust-single')
 * // rustSingleSupports might be: Set { 'pdf', 'docx', 'txt' }
 */
export function getFrameworkCapabilities(
  data: AggregatedBenchmarkData
): Map<string, Set<string>> {
  const capabilities = new Map<string, Set<string>>()

  // Iterate through each framework-mode combination
  Object.entries(data.by_framework_mode).forEach(([key, frameworkData]) => {
    const supportedFileTypes = new Set<string>()

    // Check each file type for this framework-mode
    Object.entries(frameworkData.by_file_type).forEach(([fileType, metrics]) => {
      // Check both OCR modes - if either mode works, the file type is supported
      const noOcrSupported =
        metrics.no_ocr && metrics.no_ocr.duration.p50 < UNSUPPORTED_THRESHOLD_MS
      const withOcrSupported =
        metrics.with_ocr && metrics.with_ocr.duration.p50 < UNSUPPORTED_THRESHOLD_MS

      // If at least one OCR mode works, consider the file type supported
      if (noOcrSupported || withOcrSupported) {
        supportedFileTypes.add(fileType)
      }
    })

    capabilities.set(key, supportedFileTypes)
  })

  return capabilities
}

/**
 * Filters a list of framework keys to only include those that support a specific file type.
 * This is defensive - if capabilities are not found for a framework, it will be included
 * (opt-in filtering).
 *
 * @param frameworkKeys - Array of framework-mode identifiers to filter
 * @param fileType - The file type to check support for (e.g., "pdf", "jpg")
 * @param capabilities - Map of framework capabilities (from getFrameworkCapabilities)
 * @returns Filtered array containing only frameworks that support the file type
 *
 * @example
 * const capabilities = getFrameworkCapabilities(data)
 * const allFrameworks = ['rust-single', 'pandoc-single', 'python-single']
 * const jpgFrameworks = filterFrameworksByFileType(allFrameworks, 'jpg', capabilities)
 * // jpgFrameworks might be: ['rust-single', 'python-single'] (excluding pandoc)
 */
export function filterFrameworksByFileType(
  frameworkKeys: string[],
  fileType: string,
  capabilities: Map<string, Set<string>>
): string[] {
  return frameworkKeys.filter((key) => {
    const supportedTypes = capabilities.get(key)

    // Defensive: if we don't have capability data for this framework, include it
    if (!supportedTypes) {
      return true
    }

    // Check if this framework supports the file type
    return supportedTypes.has(fileType)
  })
}

/**
 * Gets the count of frameworks that support a specific file type.
 *
 * @param fileType - The file type to check
 * @param capabilities - Map of framework capabilities
 * @returns Object containing the count of supporting frameworks and total frameworks
 */
export function getFileTypeSupport(
  fileType: string,
  capabilities: Map<string, Set<string>>
): { supporting: number; total: number } {
  const total = capabilities.size
  let supporting = 0

  capabilities.forEach((supportedTypes) => {
    if (supportedTypes.has(fileType)) {
      supporting++
    }
  })

  return { supporting, total }
}
