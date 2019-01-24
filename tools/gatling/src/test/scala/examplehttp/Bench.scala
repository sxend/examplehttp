package examplehttp

import com.typesafe.config.ConfigFactory
import io.gatling.core.Predef._
import io.gatling.http.Predef._

import scala.concurrent.duration._

class Bench extends Simulation {
  val config = ConfigFactory.load
  val httpConf = http
    .baseURL(s"http://${config.getString("target-address")}")
    .userAgentHeader("Mozilla/5.0 (iPhone; CPU iPhone OS 11_0 like Mac OS X) AppleWebKit/604.1.38 (KHTML, like Gecko) Version/11.0 Mobile/15A372 Safari/604.1")

  val scn = scenario("HealthSimulation")
    .exec(http("request").get("/"))

  setUp(
    scn.inject(
      constantUsersPerSec(600) during (60 seconds)
    )
  ).protocols(httpConf)

}