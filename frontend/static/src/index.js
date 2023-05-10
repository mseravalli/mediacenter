function create_connect_ajax(country_code) {
  return function connect() {
    $.ajax({
      url: "http://localhost:8080/connect",
      data: {
        location: country_code
      },
      success: function( result ) {
        console.log("Connection in progress.");
      },
      error: function() {
        console.log("Connection failed.");
      }
    });
  };
}

function create_is_connected_ajax(country_code) {
  return $.ajax({
    url: "http://localhost:8080/is_connected",
    data: {
      location: country_code
    },
    success: function( result ) {
      $("#vpn_"+country_code+"_img").removeClass("green_background");
      $("#vpn_"+country_code+"_img").removeClass("yellow_background");
      $("#vpn_"+country_code+"_img").removeClass("red_background");
      if (result == "Already connected to "+country_code+".") {
        $("#vpn_"+country_code+"_img").addClass("green_background");
      } else if (result == "Connection to "+country_code+" in progress.") {
        $("#vpn_"+country_code+"_img").addClass("yellow_background");
      } else {
        $("#vpn_"+country_code+"_img").addClass("red_background");
      }
    },
    error: function() {
      $("#vpn_"+country_code+"_img").addClass("red_background");
    }
  });
}

$(document).ready(function(){
  $("#vpn_IT").click(create_connect_ajax("IT"));
  $("#vpn_DE").click(create_connect_ajax("DE"));
});

function poll(){
  create_is_connected_ajax("IT");
  create_is_connected_ajax("DE");
}
setInterval(poll, 1000);

