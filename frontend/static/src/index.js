$(document).ready(function(){
  $("#vpn_italy").click(
    function connect(){
      $.ajax({
        url: "http://localhost:8080/connect",
        data: {
          location: "IT"
        },
        success: function( result ) {
          console.log("Connection in progress.");
        },
        error: function() {
          console.log("Connection failed.");
        }
      });
    }
  );
  $("#vpn_germany").click(
    function connect(){
      $.ajax({
        url: "http://localhost:8080/connect",
        data: {
          location: "DE"
        },
        success: function( result ) {
          console.log("Connection in progress.");
        },
        error: function() {
          console.log("Connection failed.");
        }
      });
    }
  );
});

function poll(){
  $.ajax({
    url: "http://localhost:8080/is_connected",
    data: {
      location: "IT"
    },
    success: function( result ) {
      $("#vpn_italy_img").removeClass("green_background");
      $("#vpn_italy_img").removeClass("yellow_background");
      $("#vpn_italy_img").removeClass("red_background");
      if (result == "Already connected to IT.") {
        $("#vpn_italy_img").addClass("green_background");
        console.log("green");
      } else if (result == "Connection to IT in progress.") {
        $("#vpn_italy_img").addClass("yellow_background");
        console.log("yellow");
      } else {
        $("#vpn_italy_img").addClass("red_background");
        console.log("red");
      }
    },
    error: function() {
      $("#vpn_italy_img").addClass("red_background");
    }
  });
  $.ajax({
    url: "http://localhost:8080/is_connected",
    data: {
      location: "DE"
    },
    success: function( result ) {
      $("#vpn_germany_img").removeClass("green_background");
      $("#vpn_germany_img").removeClass("yellow_background");
      $("#vpn_germany_img").removeClass("red_background");
      if (result == "Already connected to DE.") {
        $("#vpn_germany_img").addClass("green_background");
      } else if (result == "Connection to DE in progress.") {
        $("#vpn_germany_img").addClass("yellow_background");
      } else {
        $("#vpn_germany_img").addClass("red_background");
      }
    },
    error: function() {
      $("#vpn_germany_img").addClass("red_background");
    }
  });
}
setInterval(poll, 1000);

