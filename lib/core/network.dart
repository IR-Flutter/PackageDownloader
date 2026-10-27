import 'dart:io';

class MirrorBenchmarkResult {
  final String mirrorId;
  final bool isReachable;
  final int latencyMs;
  final String? error;

  MirrorBenchmarkResult({
    required this.mirrorId,
    required this.isReachable,
    required this.latencyMs,
    this.error,
  });
}

class NetworkBenchmark {
  static Future<MirrorBenchmarkResult> testMirror(
    String mirrorId,
    String url, {
    Duration timeout = const Duration(seconds: 5),
  }) async {
    final client = HttpClient();
    client.connectionTimeout = timeout;

    final stopwatch = Stopwatch()..start();
    try {
      final uri = Uri.parse(url);
      final request = await client.getUrl(uri);
      final response = await request.close().timeout(timeout);
      stopwatch.stop();

      // Drain response
      await response.drain();

      final reachable = response.statusCode >= 200 && response.statusCode < 400;
      return MirrorBenchmarkResult(
        mirrorId: mirrorId,
        isReachable: reachable,
        latencyMs: stopwatch.elapsedMilliseconds,
      );
    } catch (e) {
      stopwatch.stop();
      return MirrorBenchmarkResult(
        mirrorId: mirrorId,
        isReachable: false,
        latencyMs: stopwatch.elapsedMilliseconds,
        error: e.toString(),
      );
    } finally {
      client.close(force: true);
    }
  }
}
