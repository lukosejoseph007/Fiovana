// Vector Operations Service
import { apiClient } from '../api'
import {
  ClusterResult,
  ApiResponse
} from '../types'

export class VectorOperationsService {
  /**
   * Perform advanced vector clustering
   */
  async performVectorClustering(
    vectors: number[][],
    clusteringOptions?: unknown
  ): Promise<ApiResponse<ClusterResult[]>> {
    return apiClient.invoke('perform_vector_clustering', {
      vectors,
      options: clusteringOptions || {}
    })
  }

  /**
   * Calculate vector similarities
   */
  async calculateVectorSimilarities(
    sourceVector: number[],
    targetVectors: number[][],
    similarityMetric?: string
  ): Promise<ApiResponse<number[]>> {
    return apiClient.invoke('calculate_vector_similarities', {
      source_vector: sourceVector,
      target_vectors: targetVectors,
      similarity_metric: similarityMetric || 'cosine'
    })
  }

  /**
   * Perform vector dimensionality reduction
   */
  async reduceDimensionality(
    vectors: number[][],
    targetDimensions: number,
    method?: string
  ): Promise<ApiResponse<number[][]>> {
    return apiClient.invoke('reduce_vector_dimensionality', {
      vectors,
      target_dimensions: targetDimensions,
      method: method || 'pca'
    })
  }

  /**
   * Perform vector space analysis
   */
  async analyzeVectorSpace(
    vectors: number[][],
    analysisOptions?: unknown
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_vector_space', {
      vectors,
      options: analysisOptions || {}
    })
  }

  /**
   * Find vector outliers
   */
  async findVectorOutliers(
    vectors: number[][],
    outlierThreshold?: number
  ): Promise<ApiResponse<number[]>> {
    return apiClient.invoke('find_vector_outliers', {
      vectors,
      outlier_threshold: outlierThreshold || 2.0
    })
  }

  /**
   * Perform hierarchical clustering
   */
  async performHierarchicalClustering(
    vectors: number[][],
    linkageMethod?: string
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('perform_hierarchical_clustering', {
      vectors,
      linkage_method: linkageMethod || 'ward'
    })
  }

  /**
   * Perform K-means clustering
   */
  async performKMeansClustering(
    vectors: number[][],
    k: number,
    kmeansOptions?: unknown
  ): Promise<ApiResponse<ClusterResult[]>> {
    return apiClient.invoke('perform_kmeans_clustering', {
      vectors,
      k,
      options: kmeansOptions || {}
    })
  }

  /**
   * Perform DBSCAN clustering
   */
  async performDBSCANClustering(
    vectors: number[][],
    eps?: number,
    minSamples?: number
  ): Promise<ApiResponse<ClusterResult[]>> {
    return apiClient.invoke('perform_dbscan_clustering', {
      vectors,
      eps: eps || 0.5,
      min_samples: minSamples || 5
    })
  }

  /**
   * Calculate vector centroids
   */
  async calculateCentroids(clusters: ClusterResult[]): Promise<ApiResponse<number[][]>> {
    return apiClient.invoke('calculate_cluster_centroids', {
      clusters
    })
  }

  /**
   * Perform vector interpolation
   */
  async interpolateVectors(
    vector1: number[],
    vector2: number[],
    interpolationFactor: number
  ): Promise<ApiResponse<number[]>> {
    return apiClient.invoke('interpolate_vectors', {
      vector1,
      vector2,
      interpolation_factor: interpolationFactor
    })
  }

  /**
   * Find nearest neighbors
   */
  async findNearestNeighbors(
    queryVector: number[],
    candidateVectors: number[][],
    k: number,
    distanceMetric?: string
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('find_nearest_neighbors', {
      query_vector: queryVector,
      candidate_vectors: candidateVectors,
      k,
      distance_metric: distanceMetric || 'euclidean'
    })
  }

  /**
   * Perform vector normalization
   */
  async normalizeVectors(
    vectors: number[][],
    normalizationMethod?: string
  ): Promise<ApiResponse<number[][]>> {
    return apiClient.invoke('normalize_vectors', {
      vectors,
      normalization_method: normalizationMethod || 'l2'
    })
  }

  /**
   * Calculate vector statistics
   */
  async calculateVectorStatistics(vectors: number[][]): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('calculate_vector_statistics', {
      vectors
    })
  }

  /**
   * Perform vector transformation
   */
  async transformVectors(
    vectors: number[][],
    transformationMatrix: number[][]
  ): Promise<ApiResponse<number[][]>> {
    return apiClient.invoke('transform_vectors', {
      vectors,
      transformation_matrix: transformationMatrix
    })
  }

  /**
   * Analyze cluster quality
   */
  async analyzeClusterQuality(
    vectors: number[][],
    clusters: ClusterResult[]
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_cluster_quality', {
      vectors,
      clusters
    })
  }

  /**
   * Perform vector compression
   */
  async compressVectors(
    vectors: number[][],
    compressionRatio: number
  ): Promise<ApiResponse<number[][]>> {
    return apiClient.invoke('compress_vectors', {
      vectors,
      compression_ratio: compressionRatio
    })
  }

  /**
   * Build vector index for fast similarity search
   */
  async buildVectorIndex(
    vectors: number[][],
    indexType?: string,
    indexOptions?: unknown
  ): Promise<ApiResponse<string>> {
    return apiClient.invoke('build_vector_index', {
      vectors,
      index_type: indexType || 'hnsw',
      options: indexOptions || {}
    })
  }

  /**
   * Search vector index
   */
  async searchVectorIndex(
    indexId: string,
    queryVector: number[],
    k: number
  ): Promise<ApiResponse<unknown[]>> {
    return apiClient.invoke('search_vector_index', {
      index_id: indexId,
      query_vector: queryVector,
      k
    })
  }

  /**
   * Perform vector arithmetic operations
   */
  async performVectorArithmetic(
    operation: string,
    vectors: number[][]
  ): Promise<ApiResponse<number[]>> {
    return apiClient.invoke('perform_vector_arithmetic', {
      operation,
      vectors
    })
  }

  /**
   * Analyze vector density
   */
  async analyzeVectorDensity(
    vectors: number[][],
    densityOptions?: unknown
  ): Promise<ApiResponse<unknown>> {
    return apiClient.invoke('analyze_vector_density', {
      vectors,
      options: densityOptions || {}
    })
  }
}

export const vectorOperationsService = new VectorOperationsService()