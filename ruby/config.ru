#!/usr/bin/env ruby
# encoding: utf-8

require "json"
require "syro"
require "ohm"
require "redic"

require_relative "models"

Ohm.redis = Redic.new("redis://127.0.0.1:6379/1")

FRONTENT_DIR = "../frontend"
SESSION_SECRET = ENV.fetch("RACK_SESSION_SECRET", "87998b9378")

class API < Syro::Deck
  def log *args
    $stderr.puts "@@@@ %s" % args.inspect
  end

  def default_headers
    { "Content-Type" => "application/json" }
  end

  def json(object)
    res.write object.to_json
  end

  def unauthorized(reason = "Unauthorized")
    req.session["user-id"] = ""
    res.status = 401
    json(authorized: false, reason: reason)
    halt(res.finish)
  end

  def notfound
    res.status = 404
    json(reason: "Track not found")
    halt(res.finish)
  end

  def current_user
    @current_user ||= begin
                        user_id = req.session["user-id"]
                        $stderr.puts "Got User id from cookie: #{user_id.inspect}"

                        if user_id.nil?
                          user_id = req.params["user_id"]
                          $stderr.puts "Got User id from parameter: #{user_id.inspect}"
                          user_id = user_id.to_i unless user_id.nil?
                        end

                        user = fetch_user_or_create(user_id)

                        $stderr.puts "User: #{user.inspect}"

                        if user.nil?
                          unauthorized("Failed to find user") and return
                        end

                        set_user_cookie(user)
                        user
                      end
  end

  def set_user_cookie(user)
    $stderr.puts "User cookie set to #{user.name}"
    req.session["user-id"] = user.name
    res.set_cookie("user-id", user.name.to_s)
  end

  def new_random_user
    retries = 5
    begin
      user_id = rand(100_000_000).to_s
      u = User.create(name: user_id)
      return u
    rescue Ohm::UniqueIndexViolation
      retries -= 1

      if retries == 0
        return nil
      else
        retry
      end
    end
  end

  def fetch_user_or_create(user_id)
    $stdout.puts "fetch_user_or_create with id=#{user_id.inspect}"

    if user_id.nil?
      return new_random_user
    end

    user = User.with(:name, user_id)
    if user.nil?
      user = User.create(name: user_id)
    end

    user
  end
end


app = Syro.new(API) {
  on("time") {
    on("new") {
      # Verify user first
      user = current_user

      post {
        start = req.params["start"].to_i
        stop = req.params["stop"].to_i
        if start == 0 || stop == 0
          json(error: "Start and stop parameters required.")
        else
          track = TimeTrack.create(start: start,
                                   stop: stop,
                                   user: user)
          json(track)
        end
      }
    }

    on(:track_id) {
      get {
        id = inbox[:track_id].to_i
        track = TimeTrack[id]
        if track.nil?
          notfound
        end

        if track.user != current_user
          unauthorized("Not allowed")
        end

        json(track)
      }
    }

    get {
      json(current_user.tracks.to_a)
    }
  }
}

if ENV.fetch("RACK_ENV") == "production"
  mount_path = "/legacy"
else
  mount_path = "/"
end

map mount_path do
  use Rack::MethodOverride
  use Rack::Session::Cookie, secret: SESSION_SECRET, httponly: false
  use Rack::CommonLogger, $stdout

  map "/api" do
    run(app)
  end

  run Rack::File.new(FRONTENT_DIR)
end
