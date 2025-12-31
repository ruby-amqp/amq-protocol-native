# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("amq_protocol_native/amq_protocol_native") do |r|
  r.profile = ENV.fetch("RB_SYS_CARGO_PROFILE", :release).to_sym
end
