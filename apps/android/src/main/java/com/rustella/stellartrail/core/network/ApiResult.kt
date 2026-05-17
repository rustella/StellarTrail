package com.rustella.stellartrail.core.network

sealed interface ApiResult<out T> {
    data class Success<T>(val value: T) : ApiResult<T>
    data class Failure(val throwable: Throwable) : ApiResult<Nothing>
}

suspend fun <T> runApi(block: suspend () -> T): ApiResult<T> = try {
    ApiResult.Success(block())
} catch (throwable: Throwable) {
    ApiResult.Failure(throwable)
}
