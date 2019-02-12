import akka.actor.ActorSystem
import akka.http.scaladsl.Http
import akka.http.scaladsl.model.HttpRequest
import akka.http.scaladsl.server.{RequestContext, Route}
import akka.stream.ActorMaterializer
import akka.http.scaladsl.server.Directives._
import akka.stream.scaladsl._
import akka.http.scaladsl.marshallers.sprayjson.SprayJsonSupport
import com.typesafe.config.ConfigFactory
import spray.json._

import scala.concurrent.{ExecutionContext, Future, Promise}
import scala.concurrent.duration._

object Main extends SprayJsonSupport with DefaultJsonProtocol {
  implicit val extFormat = jsonFormat1(Ext.apply)
  implicit val headerFormat = jsonFormat2(Header.apply)
  implicit val reqFormat = jsonFormat4(Request.apply)
  implicit val mFormat = jsonFormat2(Message.apply)
  implicit val printer = PrettyPrinter
  val config = ConfigFactory.load
  val port = config.getInt("port")
  val delay = config.getInt("delay")
  implicit val system = ActorSystem()
  implicit val materializer = ActorMaterializer()
  implicit val executionContext: ExecutionContext = system.dispatcher
  def main(args: Array[String]): Unit = {
    Http().bindAndHandleAsync(Route.asyncHandler(route), "0.0.0.0", port)
  }
  def route = extractRequestContext { ctx =>
    complete(handle(ctx))
  }
  def handle(ctx: RequestContext): Future[Message] = {
    if (delay > 0) {
      val promise = Promise[Message]()
      system.scheduler.scheduleOnce(delay.millis, new Runnable {
        override def run(): Unit = promise.success(message(ctx))
      })
      promise.future
    } else Future.successful(message(ctx))
  }
  def message(ctx: RequestContext) = Message(
    request = Request(
      version = ctx.request.protocol.value,
      method = ctx.request.method.value,
      path = ctx.request.uri.path.toString(),
      headers = ctx.request.headers.map(header => Header(header.name(), header.value())).toList,
    ),
    ext = Ext(
      processThread = s"${Thread.currentThread().getName}-${Thread.currentThread().getId}"
    )
  )
  case class Message(
                    request: Request,
                    ext: Ext
                    )
  case class Request(
                    version: String,
                    method: String,
                    path: String,
                    headers: List[Header]
                    )
  case class Header(
                   name: String,
                   value: String
                   )
  case class Ext(processThread: String)

}