package examplehttp

import com.typesafe.config.ConfigFactory
import io.gatling.core.Predef._
import io.gatling.http.Predef._

import scala.concurrent.duration._

class Bench extends Simulation {
  val config = ConfigFactory.load
  val targetName = config.getString("target-name")
  val targetConfig = config.getConfig(s"targets.$targetName")
  val url = s"http://${targetConfig.getString("host")}:${targetConfig.getInt("port")}"
  val httpConf = http
    .baseURL(url)
    .userAgentHeader("Mozilla/5.0 (iPhone; CPU iPhone OS 11_0 like Mac OS X) AppleWebKit/604.1.38 (KHTML, like Gecko) Version/11.0 Mobile/15A372 Safari/604.1")

  val scn = scenario(s"$targetName-simulation")
    .exec(http(url).get("/"))

  setUp(
    scn.inject(
      rampUsersPerSec(1) to (config.getInt("constant-users")) during ((config.getInt("during") / 2) seconds),
      constantUsersPerSec(config.getInt("constant-users")) during (config.getInt("during") seconds)
    )
  ).protocols(httpConf)

}