@rem Gradle startup script for Windows
@echo off
set DIRNAME=%~dp0
if "%JAVA_HOME%" == "" goto noJavaHome
set JAVACMD=%JAVA_HOME%in\java.exe
goto run
:noJavaHome
set JAVACMD=java.exe
:run
"%JAVACMD%" -classpath "%DIRNAME%gradle\wrapper\gradle-wrapper.jar" org.gradle.wrapper.GradleWrapperMain %*
