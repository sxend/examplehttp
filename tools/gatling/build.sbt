name := "examplehttp-gatling"

version := "0.1"

scalaVersion := "2.12.8"

libraryDependencies ++= Seq(
  "io.gatling" % "gatling-test-framework" % "2.3.0" % "test",
  "io.gatling.highcharts" % "gatling-charts-highcharts" % "2.3.0" % "test"
)

enablePlugins(GatlingPlugin)